use kornia_core::SafeTensorType;
use kornia_image::{Image, ImageError};

/// Normalize an image using the mean and standard deviation.
///
/// The formula for normalizing an image is:
///
/// (image - mean) / std
///
/// Each channel is normalized independently.
///
/// # Arguments
///
/// * `src` - The input image of shape (height, width, channels).
/// * `dst` - The output image of shape (height, width, channels).
/// * `mean` - The mean value for each channel.
/// * `std` - The standard deviation for each channel.
///
/// # Returns
///
/// The normalized image of shape (height, width, channels).
///
/// # Example
///
/// ```
/// use kornia::image::{Image, ImageSize};
/// use kornia::imgproc::normalize::normalize_mean_std;
///
/// let image_data = vec![0f32, 1.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 1.0, 2.0, 3.0];
/// let image = Image::<f32, 3>::new(
///   ImageSize {
///     width: 2,
///     height: 2,
///   },
///   image_data,
/// )
/// .unwrap();
///
/// let mut image_normalized = Image::<f32, 3>::from_size_val(image.size(), 0.0).unwrap();
///
/// normalize_mean_std(
///     &image,
///     &mut image_normalized,
///     &[0.5, 1.0, 0.5],
///     &[1.0, 1.0, 1.0],
/// )
/// .unwrap();
///
/// assert_eq!(image_normalized.num_channels(), 3);
/// assert_eq!(image_normalized.size().width, 2);
/// assert_eq!(image_normalized.size().height, 2);
/// ```
pub fn normalize_mean_std<T, const CHANNELS: usize>(
    src: &Image<T, CHANNELS>,
    dst: &mut Image<T, CHANNELS>,
    mean: &[T; CHANNELS],
    std: &[T; CHANNELS],
) -> Result<(), ImageError>
where
    T: num_traits::Float
        + num_traits::FromPrimitive
        + std::fmt::Debug
        + Send
        + Sync
        + Copy
        + SafeTensorType,
{
    if src.size() != dst.size() {
        return Err(ImageError::InvalidImageSize(
            src.size().width,
            src.size().height,
            dst.size().width,
            dst.size().height,
        ));
    }

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

    ndarray::Zip::from(dst_data.rows_mut())
        .and(src_data.rows())
        .par_for_each(|mut out, inp| {
            for i in 0..CHANNELS {
                out[i] = (inp[i] - mean[i]) / std[i];
            }
        });

    // copy the data back to the dst image
    dst.as_slice_mut()
        .copy_from_slice(dst_data.as_slice().unwrap());

    Ok(())
}

/// Find the minimum and maximum values in an image.
///
/// # Arguments
///
/// * `src` - The input image of shape (height, width, channels).
/// * `dst` - The output image of shape (height, width, channels).
///
/// # Returns
///
/// A tuple containing the minimum and maximum values in the image.
///
/// # Errors
///
/// If the image data is not initialized, an error is returned.
///
/// # Example
///
/// ```
/// use kornia::image::{Image, ImageSize};
/// use kornia::imgproc::normalize::find_min_max;
///
/// let image_data = vec![0u8, 1, 0, 1, 2, 3, 0, 1, 0, 1, 2, 3];
/// let image = Image::<u8, 3>::new(
///   ImageSize {
///     width: 2,
///     height: 2,
///   },
///   image_data,
/// )
/// .unwrap();
///
/// let (min, max) = find_min_max(&image).unwrap();
/// assert_eq!(min, 0);
/// assert_eq!(max, 3);
/// ```
pub fn find_min_max<T, const CHANNELS: usize>(
    image: &Image<T, CHANNELS>,
) -> Result<(T, T), ImageError>
where
    T: SafeTensorType,
    //T: PartialOrd + Copy,
{
    // get the first element in the image
    let first_element = match image.as_slice().iter().next() {
        Some(x) => x,
        None => return Err(ImageError::ImageDataNotInitialized),
    };

    let mut min = first_element;
    let mut max = first_element;

    for x in image.as_slice().iter() {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }

    Ok((*min, *max))
}

/// Normalize an image using the minimum and maximum values.
///
/// The formula for normalizing an image is:
///
/// (image - min) * (max - min) / (max_val - min_val) + min
///
/// Each channel is normalized independently.
///
/// # Arguments
///
/// * `src` - The input image of shape (height, width, channels).
/// * `dst` - The output image of shape (height, width, channels).
/// * `min` - The minimum value for each channel.
/// * `max` - The maximum value for each channel.
///
/// # Returns
///
/// The normalized image of shape (height, width, channels).
///
/// # Example
///
/// ```
/// use kornia::image::{Image, ImageSize};
/// use kornia::imgproc::normalize::normalize_min_max;
///
/// let image_data = vec![0.0f32, 1.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 1.0, 2.0, 3.0];
/// let image = Image::<f32, 3>::new(
///   ImageSize {
///     width: 2,
///     height: 2,
///   },
///   image_data,
/// )
/// .unwrap();
///
/// let mut image_normalized = Image::<f32, 3>::from_size_val(image.size(), 0.0).unwrap();
///
/// normalize_min_max(&image, &mut image_normalized, 0.0, 1.0).unwrap();
///
/// assert_eq!(image_normalized.num_channels(), 3);
/// assert_eq!(image_normalized.size().width, 2);
/// assert_eq!(image_normalized.size().height, 2);
/// ```
pub fn normalize_min_max<T, const CHANNELS: usize>(
    src: &Image<T, CHANNELS>,
    dst: &mut Image<T, CHANNELS>,
    min: T,
    max: T,
) -> Result<(), ImageError>
where
    T: num_traits::Float
        + num_traits::FromPrimitive
        + std::fmt::Debug
        + Send
        + Sync
        + Copy
        + Default
        + SafeTensorType,
{
    if src.size() != dst.size() {
        return Err(ImageError::InvalidImageSize(
            src.size().width,
            src.size().height,
            dst.size().width,
            dst.size().height,
        ));
    }

    let (min_val, max_val) = find_min_max(src)?;

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

    ndarray::Zip::from(dst_data.rows_mut())
        .and(src_data.rows())
        .par_for_each(|mut out, inp| {
            for i in 0..CHANNELS {
                out[i] = (inp[i] - min_val) * (max - min) / (max_val - min_val) + min;
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
    fn normalize_mean_std() -> Result<(), ImageError> {
        let image_data = vec![
            0.0f32, 1.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 1.0, 2.0, 3.0,
        ];

        let image_expected = [
            -0.5f32, 0.0, -0.5, 0.5, 1.0, 2.5, -0.5, 0.0, -0.5, 0.5, 1.0, 2.5,
        ];

        let image = Image::<f32, 3>::new(
            ImageSize {
                width: 2,
                height: 2,
            },
            image_data,
        )?;

        let mean = [0.5, 1.0, 0.5];
        let std = [1.0, 1.0, 1.0];

        let mut normalized = Image::<f32, 3>::from_size_val(image.size(), 0.0)?;

        super::normalize_mean_std(&image, &mut normalized, &mean, &std)?;

        assert_eq!(normalized.num_channels(), 3);
        assert_eq!(normalized.size().width, 2);
        assert_eq!(normalized.size().height, 2);

        normalized
            .as_slice()
            .iter()
            .zip(image_expected.iter())
            .for_each(|(a, b)| {
                assert!((a - b).abs() < 1e-6);
            });

        Ok(())
    }

    #[test]
    fn find_min_max() -> Result<(), ImageError> {
        let image_data = vec![0u8, 1, 0, 1, 2, 3, 0, 1, 0, 1, 2, 3];
        let image = Image::<u8, 3>::new(
            ImageSize {
                width: 2,
                height: 2,
            },
            image_data,
        )?;

        let (min, max) = super::find_min_max(&image)?;

        assert_eq!(min, 0);
        assert_eq!(max, 3);

        Ok(())
    }

    #[test]
    fn normalize_min_max() -> Result<(), ImageError> {
        let image_data = vec![
            0.0f32, 1.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 0.0, 1.0, 2.0, 3.0,
        ];

        let image_expected = [
            0.0f32, 0.33333334, 0.0, 0.33333334, 0.6666667, 1.0, 0.0, 0.33333334, 0.0, 0.33333334,
            0.6666667, 1.0,
        ];

        let image = Image::<f32, 3>::new(
            ImageSize {
                width: 2,
                height: 2,
            },
            image_data,
        )?;

        let mut normalized = Image::<f32, 3>::from_size_val(image.size(), 0.0)?;

        super::normalize_min_max(&image, &mut normalized, 0.0, 1.0)?;

        assert_eq!(normalized.num_channels(), 3);
        assert_eq!(normalized.size().width, 2);
        assert_eq!(normalized.size().height, 2);

        normalized
            .as_slice()
            .iter()
            .zip(image_expected.iter())
            .for_each(|(a, b)| {
                assert!((a - b).abs() < 1e-6);
            });

        Ok(())
    }
}
