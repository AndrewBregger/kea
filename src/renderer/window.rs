use crate::app::Config;
use glutin::ContextCurrentState;

// Needed glutin modules and structures
pub use glutin::{
    event,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window, NotCurrent, PossiblyCurrent, WindowedContext,
};

// needed for error handling
pub use glutin::{ContextError, CreationError};

// needed for size
pub use glutin::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};

#[derive(thiserror::Error, Debug)]
pub enum WindowError {
    #[error("Failed to create rendering context: {0}")]
    ContextError(ContextError),
    #[error("Failed to create window: {0}")]
    NoWindow(CreationError),
}

// Uses the entire Result path so result is not redeclared in this scope.
pub type Result<T> = ::std::result::Result<T, WindowError>;

#[derive(Debug)]
pub struct Window<Context: ContextCurrentState> {
    // The main display window. Eventually this window could be owned by an other window.
    // This will allow for integration of working project between multiple windows.
    context: WindowedContext<Context>,

    // Is this window the active window of the user.
    focused: bool,
}

// @TODO: extend window building to platform specific window builders to
//        allow for control over window properties.
//        I.E. Use glutin::os::unit::WindowBuilderExt to build linxu/unix windows
//              It gives access to other properties such as gtk themes.
//        e.t.c
//
//        For now I am only using the generic window builder.

impl Window<NotCurrent> {

    // Change size of a WindowConfig or Config object so the window
    // is created to some properties specified by the user.
    // A config created by loading some config file (rem.config or whatever)
    pub fn new<T>(event_loop: &EventLoop<T>, size: LogicalSize<f32>, title: &str, config: &Config) -> Result<Self> {
        let context = Self::build_window(event_loop, size, title, config)?;
        Ok(Self {
            context,
            focused: true,
        })
    }


    #[cfg(any(target_os = "linux", target_os = "windows"))]
    fn build_window<T>(
        event_loop: &EventLoop<T>,
        size: LogicalSize<f32>,
        title: &str,
        _config: &Config
    ) -> Result<WindowedContext<NotCurrent>> {
        use glutin::{window::WindowBuilder, ContextBuilder};

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .with_resizable(true)
            // for now
            .with_decorations(true);
        // test right now
        // .with_transparency(true);
        // @TODO: Window icon.
        // .window_window_icon(???)

        let context = ContextBuilder::new()
            // these should be checked or passed an not assumed. {
            .with_gl_debug_flag(true)
            .with_gl_robustness(glutin::Robustness::TryRobustLoseContextOnReset)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_double_buffer(Some(true))
            .with_srgb(true)
            // .with_vsync(true)
            // }
            .build_windowed(window, event_loop)
            .map_err(|e| WindowError::NoWindow(e))?;

        Ok(context)
    }

    #[cfg(target_os = "macos")]
    fn build_window<T>(
        event_loop: &EventLoop<T>,
        size: LogicalSize<f32>,
        title: &str,
        _config: &Config
    ) -> Result<WindowedContext<NotCurrent>> {
        // use super::glutin::platform::macos::WindowBuilderExtMacOS;
        use super::glutin::{window::WindowBuilder, ContextBuilder};

        let windowbuilder = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .with_resizable(true)
            // for now
            .with_decorations(true);
        // test right now
        // .with_transparency(true);

        let context = ContextBuilder::new()
            // these should be checked or passed an not assumed. {
            .with_gl_debug_flag(true)
            .with_gl_robustness(glutin::Robustness::TryRobustLoseContextOnReset)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 0)))
            .with_double_buffer(Some(true))
            .with_srgb(true)
            // .with_vsync(true)
            // }
            .build_windowed(windowbuilder, event_loop)
            .map_err(|e| WindowError::NoWindow(e))?;

        Ok(context)
    }

    pub fn make_current(self) -> Result<Window<PossiblyCurrent>> {
        let Window { context, focused} = self;
        let context = unsafe { context.make_current().map_err(|(_, e)| WindowError::ContextError(e))? };
        
        Ok(Window::<PossiblyCurrent> {
            context,
            focused
        })
    }
}

impl Window<PossiblyCurrent> {
    pub fn swap_buffers(&self) {
        self.context.swap_buffers().unwrap();
    }

    // this should never fail.
    pub fn init_gl(&self) {
        gl::load_with(|s| self.context.get_proc_address(s) as *const _);
    }

    pub fn make_not_current(self) -> Result<Window<NotCurrent>> {
        let Window { context, focused} = self;
        let context = unsafe { context.make_not_current().map_err(|(_, e)| WindowError::ContextError(e))? };
        
        Ok(Window::<NotCurrent> {
            context,
            focused
        })
    }
}

impl <Context: ContextCurrentState> Window<Context> {
    // gets the dpi of the window, this can be be changed by user action
    // such as when the window is moved to a different monitor.
    // This is needed for font rendering.
    pub fn dpi_factor(&self) -> f64 {
        self.window().scale_factor()
    }

    pub fn get_size(&self) -> PhysicalSize<u32> {
        self.window().inner_size()
    }

    pub fn get_position(&self) -> PhysicalPosition<i32> {
        match self.window().inner_position() {
            Ok(size) => size,
            Err(e) => panic!("Unhandled error: {:?}", e),
        }
    }

    pub fn window(&self) -> &window::Window {
        self.context.window()
    }

    pub fn set_title(&self, title: &str) {
        self.window().set_title(title)
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_current(&self) -> bool {
        self.context.is_current()
    }
}

