use craftcad_bom::{write_bom_csv, BomTable, CsvOptions};

#[test]
fn bom_header_is_stable() {
    let bom = BomTable { rows: vec![] };
    let csv = write_bom_csv(&bom, CsvOptions { delimiter: ',' }).expect("csv");
    let text = String::from_utf8(csv).expect("utf8");
    let mut lines = text.lines();
    let header = lines.next().expect("bom+header line");
    let header = header.trim_start_matches('\u{feff}');
    assert_eq!(
        header,
        "part_id,part_name,qty,material_name,thickness,bbox_w,bbox_h,area,perimeter,grain_dir,allow_rotate,margin,kerf"
    );
}
