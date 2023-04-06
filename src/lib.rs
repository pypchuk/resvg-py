use numpy::ndarray::Array3;
use numpy::{IntoPyArray, PyArray3};
use pyo3::prelude::*;
use resvg::tiny_skia::Pixmap;
use usvg::{Size, Tree, TreeParsing};

/// SVG parsing and rendering options.
///
/// TODO(edgarmondragon): Add more options.
#[derive(Clone)]
#[pyclass]
pub struct SVGOptions {
    /// Target DPI.
    ///
    /// Impacts units conversion.
    ///
    /// Default: 96.0
    dpi: f64,

    /// Directory that will be used during relative paths resolving.
    ///
    /// Expected to be the same as the directory that contains the SVG file,
    /// but can be set to any.
    ///
    /// Default: `None
    resources_dir: Option<std::path::PathBuf>,

    /// Default viewport width to assume if there is no `viewBox` attribute and
    /// the `width` is relative.
    ///
    /// Default: 100.0
    default_width: f64,

    /// Default viewport height to assume if there is no `viewBox` attribute and
    /// the `height` is relative.
    ///
    /// Default: 100.0
    default_height: f64,
}

#[pymethods]
impl SVGOptions {
    #[new]
    #[pyo3(signature = (*, dpi = 96.0, default_width = 100.0, default_height = 100.0, resources_dir = None))]
    fn new(
        dpi: f64,
        default_width: f64,
        default_height: f64,
        resources_dir: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            dpi,
            default_width,
            default_height,
            resources_dir,
        }
    }
}

/// A Python class for rendering SVGs.
#[pyclass]
pub struct Resvg {
    options: Option<SVGOptions>,
}

#[pymethods]
impl Resvg {
    #[new]
    fn new(options: Option<SVGOptions>) -> Self {
        Self { options }
    }

    /// Renders SVG to PNG.
    ///
    /// # Arguments
    ///
    /// * `svg` - String containing SVG data.
    /// * `width` - Width of the output image.
    /// * `height` - Height of the output image.
    ///
    /// # Returns
    ///
    /// A numpy array of shape (height, width, 4) containing RGBA data.
    fn render(&self, svg: &str, width: u32, height: u32) -> RenderedImage {
        let mut pixmap = Pixmap::new(width, height).unwrap();

        let options = if let Some(options) = &self.options {
            usvg::Options {
                dpi: options.dpi,
                default_size: Size::new(options.default_width, options.default_height).unwrap(),
                resources_dir: options.resources_dir.clone(),
                ..usvg::Options::default()
            }
        } else {
            usvg::Options::default()
        };

        let tree = Tree::from_str(svg, &options).unwrap();

        resvg::render(
            &tree,
            resvg::FitTo::Original,
            tiny_skia::Transform::default(),
            pixmap.as_mut(),
        )
        .unwrap();

        RenderedImage { pixmap }
    }
}

/// Rendered image
#[pyclass]
pub struct RenderedImage {
    pixmap: Pixmap,
}

#[pymethods]
impl RenderedImage {
    /// Get the width of the image.
    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    /// Get the height of the image.
    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    /// Returns the rendered image as a numpy array.
    ///
    /// The array has shape (height, width, 4) and contains RGBA data.
    pub fn as_array<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray3<u8>> {
        let pixels = self.pixmap.pixels();

        Ok(Array3::from_shape_fn(
            ((self.height()) as usize, (self.width()) as usize, 4),
            |(y, x, c)| {
                let index = y * self.width() as usize + x;
                let pixel = &pixels[index];

                match c {
                    0 => pixel.red(),
                    1 => pixel.green(),
                    2 => pixel.blue(),
                    3 => pixel.alpha(),
                    _ => unreachable!(),
                }
            },
        )
        .into_pyarray(py))
    }

    /// TODO(edgarrmondragon): Check if this is can be used instead of as_array.
    fn as_png(&self) -> PyResult<Vec<u8>> {
        self.pixmap.encode_png().map_err(|e| {
            pyo3::exceptions::PyException::new_err(format!("Failed to encode PNG: {}", e))
        })
    }
}

/// Python bindings for resvg.
#[pymodule]
fn resvg_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<SVGOptions>()?;
    m.add_class::<RenderedImage>()?;
    m.add_class::<Resvg>()?;
    Ok(())
}
