pub async fn get_tile(layer: i32, v: i32, h: i32, style: &str) -> Result<Vec<u8>, String> {
    let file_name = format!("static/tiles/l{layer}/{v}/{h}.jpg");
    let path = std::path::Path::new(file_name.as_str());

    //let mut client = Client::connect("postgresql://postgres:mysecretpassword@127.0.0.1:5432/osm", NoTls).unwrap();
    //let res = client.query("", &[]).unwrap();

    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(format!("File '{file_name}' not found")),
    };

    //let img = image::load_from_memory(&bytes).unwrap();
    //let img = if style == "bw" { img.grayscale() } else { img };
    //let mut bytes = Vec::new();
    //img.write_to(
    //    &mut Cursor::new(&mut bytes),
    //    image::ImageOutputFormat::Jpeg(95),
    //)
    //.unwrap();

    Ok(bytes)
}
