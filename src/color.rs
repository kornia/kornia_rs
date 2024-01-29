use crate::image::Image;
use ndarray::{Array3, Zip};

// TODO: ideally we want something like this:
// let rgb: Image<u8, RGB> = load_image("image.jpg");
// let gray: Image<u8, GRAY> = image.map(|x| (76. * x[0] + 150. * x[1] + 29. * x[2]) / 255.);
// or automatically:
// let gray = Image<u8, 1>::try_from(rgb);
pub fn gray_from_rgb(image: Image) -> Image {
    let image_data = image.data;
    let mut output = Array3::<u8>::zeros(image_data.dim());

    Zip::from(output.rows_mut())
        .and(image_data.rows())
        .par_for_each(|mut out, inp| {
            assert!(inp.len() == 3);
            let r = inp[0] as f32;
            let g = inp[1] as f32;
            let b = inp[2] as f32;
            let gray = (76. * r + 150. * g + 29. * b) / 255.;

            out[0] = gray as u8;
            out[1] = gray as u8;
            out[2] = gray as u8;
        });

    Image { data: output }
}

#[cfg(test)]
mod tests {
    use crate::io::functions as F;

    #[test]
    fn gray_from_rgb() {
        let image_path = std::path::Path::new("tests/data/dog.jpeg");
        let image = F::read_image_jpeg(image_path);
        let gray = super::gray_from_rgb(image);
        assert_eq!(gray.num_channels(), 3);
        assert_eq!(gray.image_size().width, 258);
        assert_eq!(gray.image_size().height, 195);
    }
}