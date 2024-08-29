use crate::interpolation::{interpolate_pixel, meshgrid, InterpolationMode};

use kornia_image::{Image, ImageError};
use ndarray::stack;

/// flat representation of a 3x3 matrix
pub type PerspectiveMatrix = [f32; 9];

#[rustfmt::skip]
fn determinant3x3(m: &PerspectiveMatrix) -> f32 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) -
    m[1] * (m[3] * m[8] - m[5] * m[6]) +
    m[2] * (m[3] * m[7] - m[4] * m[6])
}

#[rustfmt::skip]
fn adjugate3x3(m: &PerspectiveMatrix) -> PerspectiveMatrix {
    [
        m[4] * m[8] - m[5] * m[7],  // [0, 0]
        m[2] * m[7] - m[1] * m[8],  // [0, 1]
        m[1] * m[5] - m[2] * m[4],  // [0, 2]
        m[5] * m[6] - m[3] * m[8],  // [1, 0]
        m[0] * m[8] - m[2] * m[6],  // [1, 1]
        m[2] * m[3] - m[0] * m[5],  // [1, 2]
        m[3] * m[7] - m[4] * m[6],  // [2, 0]
        m[1] * m[6] - m[0] * m[7],  // [2, 1]
        m[0] * m[4] - m[1] * m[3],  // [2, 2]
    ]
}

// TODO: use TensorError
fn inverse_perspective_matrix(m: &PerspectiveMatrix) -> Result<PerspectiveMatrix, ImageError> {
    let det = determinant3x3(m);

    if det == 0.0 {
        return Err(ImageError::CannotComputeDeterminant);
    }

    let adj = adjugate3x3(m);
    let inv_det = 1.0 / det;

    let mut inv_m = [0.0; 9];
    for i in 0..9 {
        inv_m[i] = adj[i] * inv_det;
    }

    Ok(inv_m)
}

// implement later as batched operation
fn transform_point(x: f32, y: f32, m: PerspectiveMatrix) -> (f32, f32) {
    let w = m[6] * x + m[7] * y + m[8];
    let x = (m[0] * x + m[1] * y + m[2]) / w;
    let y = (m[3] * x + m[4] * y + m[5]) / w;
    (x, y)
}

/// Applies a perspective transformation to an image.
///
/// * `src` - The input image with shape (height, width, channels).
/// * `dst` - The output image with shape (height, width, channels).
/// * `m` - The 3x3 perspective transformation matrix src -> dst.
/// * `interpolation` - The interpolation mode to use.
///
/// # Returns
///
/// The output image with shape (new_height, new_width, channels).
///
/// # Example
///
/// ```
/// use kornia::image::{Image, ImageSize};
/// use kornia::imgproc::interpolation::InterpolationMode;
/// use kornia::imgproc::warp::warp_perspective;
///
/// let src = Image::<f32, 1>::new(
///   ImageSize {
///     width: 4,
///     height: 5,
///   },
///   vec![0.0f32; 4 * 5]
/// ).unwrap();
///
/// let m = [1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0];
///
/// let mut dst = Image::<f32, 1>::from_size_val(
///   ImageSize {
///     width: 2,
///     height: 3,
///   },
///   0.0
/// ).unwrap();
///
/// warp_perspective(&src, &mut dst, &m, InterpolationMode::Bilinear).unwrap();
///
/// assert_eq!(dst.size().width, 2);
/// assert_eq!(dst.size().height, 3);
/// ```
pub fn warp_perspective<const CHANNELS: usize>(
    src: &Image<f32, CHANNELS>,
    dst: &mut Image<f32, CHANNELS>,
    m: &PerspectiveMatrix,
    interpolation: InterpolationMode,
) -> Result<(), ImageError> {
    // inverse perspective matrix
    // TODO: allow later to skip the inverse calculation if user provides it
    let inv_m = inverse_perspective_matrix(m)?;

    // create a grid of x and y coordinates for the output image
    // TODO: make this re-useable
    let x = ndarray::Array::range(0.0, dst.width() as f32, 1.0).insert_axis(ndarray::Axis(0));
    let y = ndarray::Array::range(0.0, dst.height() as f32, 1.0).insert_axis(ndarray::Axis(0));

    // create the meshgrid of x and y coordinates, arranged in a 2D grid of shape (height, width)
    let (xx, yy) = meshgrid(&x, &y);

    // stack the x and y coordinates into a single array of shape (height, width, 2)
    let xy = stack![ndarray::Axis(2), xx, yy];

    // iterate over the output image and find the corresponding position in the input image
    let src_data = unsafe {
        ndarray::ArrayView3::from_shape_ptr(
            (src.height(), src.width(), src.num_channels()),
            src.as_ptr(),
        )
    };

    let dst_data = unsafe {
        ndarray::ArrayView3::from_shape_ptr(
            (dst.height(), dst.width(), dst.num_channels()),
            dst.as_ptr(),
        )
    };
    // NOTE: might copy
    let mut dst_data = dst_data.to_owned();

    ndarray::Zip::from(xy.rows())
        .and(dst_data.rows_mut())
        .par_for_each(|uv, mut out| {
            assert_eq!(uv.len(), 2);
            let (u, v) = (uv[0], uv[1]);

            // find corresponding position in src image
            let (u_src, v_src) = transform_point(u, v, inv_m);

            // TODO: allow for multi-channel images
            // interpolate the pixel value
            let pixels = (0..src.num_channels())
                .map(|c| interpolate_pixel(&src_data, u_src, v_src, c, interpolation));

            for (c, pixel) in pixels.enumerate() {
                out[c] = pixel;
            }
        });

    // copy the data back to the dst image
    dst.as_slice_mut()
        .copy_from_slice(dst_data.as_slice().unwrap());

    Ok(())
}

#[cfg(test)]
mod tests {
    use kornia_image::{Image, ImageError, ImageSize};

    #[test]
    fn inverse_perspective_matrix() -> Result<(), ImageError> {
        let m = [1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0];
        let expected = [1.0, 0.0, 1.0, 0.0, 1.0, -1.0, 0.0, 0.0, 1.0];
        let inv_m = super::inverse_perspective_matrix(&m)?;
        assert_eq!(inv_m, expected);
        Ok(())
    }

    #[test]
    fn transform_point() {
        let m = [1.0, 0.0, -1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0];
        let (x, y) = super::transform_point(1.0, 1.0, m);
        let (x_expected, y_expected) = (0.0, 2.0);
        assert_eq!(x, x_expected);
        assert_eq!(y, y_expected);
    }

    #[test]
    fn warp_perspective_identity() -> Result<(), ImageError> {
        let image: Image<f32, 3> = Image::from_size_val(
            ImageSize {
                width: 4,
                height: 5,
            },
            0.0f32,
        )?;

        // identity matrix
        let m = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

        let new_size = ImageSize {
            width: 2,
            height: 3,
        };

        let mut image_transformed = Image::from_size_val(new_size, 0.0)?;

        super::warp_perspective(
            &image,
            &mut image_transformed,
            &m,
            super::InterpolationMode::Bilinear,
        )?;

        assert_eq!(image_transformed.num_channels(), 3);
        assert_eq!(image_transformed.size().width, 2);
        assert_eq!(image_transformed.size().height, 3);

        Ok(())
    }

    #[test]
    fn warp_perspective_hflip() -> Result<(), ImageError> {
        let image = Image::<_, 1>::new(
            ImageSize {
                width: 2,
                height: 3,
            },
            vec![0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0],
        )?;

        let image_expected = vec![1.0, 0.0, 3.0, 2.0, 5.0, 4.0];

        // flip matrix
        let m = [-1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];

        let new_size = ImageSize {
            width: 2,
            height: 3,
        };

        let mut image_transformed = Image::<_, 1>::from_size_val(new_size, 0.0)?;

        super::warp_perspective(
            &image,
            &mut image_transformed,
            &m,
            super::InterpolationMode::Bilinear,
        )?;

        assert_eq!(image_transformed.num_channels(), 1);
        assert_eq!(image_transformed.size().width, 2);
        assert_eq!(image_transformed.size().height, 3);

        assert_eq!(image_transformed.as_slice(), image_expected);

        Ok(())
    }

    #[test]
    fn test_warp_perspective_resize() -> Result<(), ImageError> {
        let image = Image::<_, 1>::new(
            ImageSize {
                width: 4,
                height: 4,
            },
            vec![
                0.0f32, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
                15.0,
            ],
        )?;

        // resize matrix (from get_perspective_transform)
        let m = [0.3333, 0.0, 0.0, 0.0, 0.3333, 0.0, 0.0, 0.0, 1.0];

        let image_expected = vec![0.0, 3.0, 12.0, 15.0];

        let new_size = ImageSize {
            width: 2,
            height: 2,
        };

        let mut image_transformed = Image::<_, 1>::from_size_val(new_size, 0.0)?;

        super::warp_perspective(
            &image,
            &mut image_transformed,
            &m,
            super::InterpolationMode::Bilinear,
        )?;

        let mut image_resized = Image::<_, 1>::from_size_val(new_size, 0.0)?;

        crate::resize::resize_native(
            &image,
            &mut image_resized,
            super::InterpolationMode::Bilinear,
        )?;

        assert_eq!(image_transformed.num_channels(), 1);
        assert_eq!(image_transformed.size().width, 2);
        assert_eq!(image_transformed.size().height, 2);

        assert_eq!(image_transformed.as_slice(), image_expected);
        assert_eq!(image_transformed.as_slice(), image_resized.as_slice());

        Ok(())
    }
}
