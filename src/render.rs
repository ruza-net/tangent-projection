use std::{collections::HashMap, mem};

use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowId},
};

use softbuffer::{Buffer, Context, Surface};

type Sfc = Surface<&'static Window, &'static Window>;

pub struct GraphicsCtx {
    soft_ctx: Context<&'static Window>,
    surfaces: HashMap<WindowId, Sfc>,
}
impl GraphicsCtx {
    pub fn new(window: &'static Window) -> Self {
        Self {
            soft_ctx: Context::new(window)
                .expect("failed to create softbuffer context"),
            surfaces: HashMap::new(),
        }
    }
    fn create_surface(&mut self, window: &Window) -> &mut Sfc {
        self.surfaces.entry(window.id()).or_insert_with(|| {
            Surface::new(&self.soft_ctx, unsafe {
                mem::transmute::<&'_ Window, &'static Window>(window)
            })
            .expect("Failed to create a softbuffer surface")
        })
    }

    pub fn draw(
        &mut self,
        window: &Window,
        f: impl FnOnce(&mut Buffer<'_, &Window, &Window>),
    ) -> Result<(), ()> {
        let sfc = self.create_surface(window);
        let PhysicalSize { width, height } = window.inner_size();
        sfc.resize(width.try_into().unwrap(), height.try_into().unwrap())
            .expect("can't resize surface");

        let mut buf = sfc.buffer_mut().unwrap();

        f(&mut buf);

        buf.present().expect("can't present buffer");

        Ok(())
    }
}
