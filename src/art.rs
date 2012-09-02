//NOTE: apparently, when looking up statics by ID, they're offset by 0x4000.
export MapTile;
export StaticTile;
export load_tiles;
export to_bitmap;
export parse_map_tile;

type MapTile = {
    header: u32,
    image: ~[u16] //TODO: Consider making Pixel a type
};

type StaticTile = {
    data_size: u16,
    trigger: u16,
    width: u16,
    height: u16,
    image: ~[u16]
};

const transparent: u16 = 0b1000000000000000;
const expected_tile_size: uint = 2048;

fn load_tiles(root_path: ~str) -> (~[(uint, MapTile)], ~[(uint, StaticTile)]) { //TODO: Find a better return type for this
    let maybe_reader: option::Option<mul_reader::MulReader> = mul_reader::reader(root_path, ~"artidx.mul", ~"art.mul");

    if option::is_none(maybe_reader) {
        io::println("Error reading art tiles");
        assert false;
    }

    let reader: mul_reader::MulReader = option::get(maybe_reader);

    let mut map_tiles: ~[(uint, MapTile)] = ~[];
    let mut static_tiles: ~[(uint, StaticTile)] = ~[];

    let mut index:uint = 0;
    while (reader.eof() != true) {
        let item: option::Option<mul_reader::MulRecord> = reader.read();
        if option::is_some(item) {
            let unwrapped: mul_reader::MulRecord = option::get(item);
            let record_header = byte_helpers::bytes_to_le_uint(vec::slice(unwrapped.data, 0, 3));
            //Apparently, these flag values represent whether something is a tile or not
            //Others are not convinced, and think that index is all that matters

            //if (record_header > 0xFFFF || record_header == 0) {
            if (index < 0x4000) {
                /*let maybe_map_tile: option::Option<MapTile> = parse_map_tile(unwrapped);
                if option::is_some(maybe_map_tile) {
                    vec::push(map_tiles, (index, maybe_map_tile.get()));
                }*/ //Skip
            } else if (index < 0x8000){
                let maybe_static_tile: option::Option<StaticTile> = parse_static_tile(unwrapped);
                if option::is_some(maybe_static_tile) {
                    vec::push(static_tiles, (index, maybe_static_tile.get()));
                }
            }
        }
        index += 1;
    }

    return (map_tiles, static_tiles);
}

//TODO: Use borrowed pointers;
fn parse_map_tile(record: mul_reader::MulRecord) -> option::Option<MapTile> { //Interestingly, pixels seem to be 565, rather than 555

    if (vec::len(record.data) != expected_tile_size) {
        return option::None;
    }

    let record_header = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 0, 3));

    let mut image: ~[u16] = ~[];
    let data_slice: ~[u8] = vec::slice(record.data, 4, vec::len(record.data));

    let mut data_pointer: uint = 0;
    for uint::range(0, 44) |i| {
        
        let slice: uint = if (i >= 22) {(44 - i) * 2} else {(i + 1) * 2};
        vec::grow(image, (22 - (slice / 2)), transparent);
        let slice_data: ~[u8] = vec::slice(data_slice, data_pointer, data_pointer + (slice * 2));
        vec::push_all(image, byte_helpers::u8vec_to_u16vec(slice_data));
        vec::grow(image, (22 - (slice / 2)), transparent);
        data_pointer += (slice * 2);
    };

    return option::Some({
        header: record_header as u32,
        image: image 
    });
}

fn parse_static_tile(record: mul_reader::MulRecord) -> option::Option<StaticTile> {
    let data_size: u16 = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 0, 1)) as u16; //Might not be size :P
    let trigger: u16 = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 2, 3)) as u16;

    let width: u16 = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 4, 5)) as u16;
    let height: u16 = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 6, 7)) as u16;

    let mut image: ~[u16] = ~[];

    if (width == 0 || height >= 1024 || height == 0 || height >= 1024) {
        io::println("Bad image dimensions found");
        return option::None;
    }

    //Offset table
    //let mut offset_table: ~[u16] = ~[];
    //TODO: Less magic numbers
    io::println("Access");
    for uint::range(0, height as uint) |index| {
        io::println("ROW");
        let offset = byte_helpers::bytes_to_le_uint(vec::slice(record.data, 8 + (index * 2), 9 + (index * 2)));
        let mut data_start = offset + 8 + (height as uint * 2);
        let mut current_row_width:uint = 0;
        //From here, read the padding length, run length, and then Run pixels
        loop {
            io::println("LOOP");
            let padding_pixels = byte_helpers::bytes_to_le_uint(vec::slice(record.data, data_start, data_start + 1));
            let run_by = byte_helpers::bytes_to_le_uint(vec::slice(record.data, data_start + 2, data_start + 3));
            if padding_pixels == 0 && run_by == 0 {
                //Fill until current row width is less than width
                io::println("BREAK");
                if current_row_width < width as uint {
                    vec::grow(image, width as uint - current_row_width, transparent);
                }
                break;
            }
            let run_data = byte_helpers::u8vec_to_u16vec(vec::slice(record.data, data_start + 4, data_start + 4 + (run_by * 2)));

            vec::grow(image, padding_pixels, transparent);
            vec::push_all(image, run_data);

            data_start += 4 + (run_by * 2);
            current_row_width += padding_pixels + (run_by * 2);
        }
        //offset_table.push(byte_helpers::bytes_to_le_uint(vec::slice(record.data, 8 + (index * 2), 9 + (index * 2)))
    };

    return option::Some({
        data_size: data_size,
        trigger: trigger,
        width: width,
        height: height,
        image: image
    });
}



fn to_bitmap(width: u32, height: u32, data: ~[u16]) -> ~[u8] { //TODO: Make this take arbitrary pixel depths
    let signature: ~[u8] = ~[0x42, 0x4D];
    let file_size: ~[u8] = byte_helpers::uint_to_le_bytes(((width * height * 2) + 14 + 40) as u64, 4);
    let reserved: ~[u8] = ~[0, 0, 0, 0];
    let data_offset: ~[u8] = byte_helpers::uint_to_le_bytes(54, 4);

    let header_size: ~[u8] = byte_helpers::uint_to_le_bytes(40, 4);
    let width_bytes: ~[u8] = byte_helpers::uint_to_le_bytes(width as u64, 4); //FIXME: should be signed?
    let height_bytes: ~[u8] = byte_helpers::uint_to_le_bytes(height as u64, 4);
    let colour_panes: ~[u8] = ~[1, 0];
    let depth: ~[u8] = ~[16, 0];
    let compression: ~[u8] = ~[0,0,0,0];
    let image_size: ~[u8] = ~[0,0,0,0];
    let horizontal_res: ~[u8] = ~[0, 0, 0, 0];
    let vertical_res: ~[u8] = ~[0, 0, 0, 0];
    let palette_count: ~[u8] = ~[0, 0, 0, 0];
    let important_colours: ~[u8] = ~[0, 0, 0, 0];

    //54 bytes so far
    //TODO: explode the image vector, iterate backwards, turn it into bytes
    let mut rows: ~[mut ~[u8]] = ~[mut];
    for uint::range(0, height as uint) |i| {
        let slice = vec::slice(data, i * (width as uint), (i+1) * (width as uint));
        let mut row: ~[u8] = ~[];
        for slice.each |sliced| {
            vec::push_all(row, byte_helpers::uint_to_le_bytes(sliced as u64, 2));
        }
        vec::push(rows, row);
    }; 
    vec::reverse(rows);
    //vec::grow(pixels, 44 * 44 * 4, 0x7f);

    return vec::concat(~[
        signature,
        file_size,
        reserved,
        data_offset,

        header_size,
        width_bytes,
        height_bytes,
        colour_panes,
        depth,
        compression,
        image_size,
        horizontal_res,
        vertical_res,
        palette_count,
        important_colours,

        vec::concat(rows)
    ]);
}
