## AVI
The `.avi`

### Usage
```rs
let data = unimplemented!();
let (avi, frames) = avi::read_avi(&data).unwrap();

for frame in frames {
    let image = image::load_from_memory(&frame.0).unwrap();
    let bytes = image.as_bytes();
}
```
