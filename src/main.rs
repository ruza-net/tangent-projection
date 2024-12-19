use std::{f32::consts::PI, mem};

use render::GraphicsCtx;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::{Window, WindowId},
};

mod render;

const GLOBE: &[u8] = include_bytes!("../earth.bmp");

pub struct App {
    title: String,
    window: Option<Window>,
    gtx: Option<GraphicsCtx>,

    scale: f32,
    cam_offset: [f32; 2],
    drag: bool,
    mouse_pos: [f32; 2],
    rot: [f32; 2],

    data_offset: u32,
    data_width: u32,
    data_height: u32,
}
impl App {
    pub fn new(
        title: String,
        data_offset: u32,
        data_width: u32,
        data_height: u32,
    ) -> Self {
        Self {
            title,
            window: None,
            gtx: None,

            scale: 1.0,
            cam_offset: [0.0, 0.0],
            drag: false,
            mouse_pos: [0.0, 0.0],
            rot: [0.0, 0.0],

            data_offset,
            data_width,
            data_height,
        }
    }

    fn redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(&self.title)
                        .with_position(LogicalPosition::new(0.0, 0.0))
                        .with_inner_size(LogicalSize::new(640.0, 320.0)),
                )
                .unwrap();

            self.window = Some(window);
        }
        if self.gtx.is_none() {
            self.gtx = Some(GraphicsCtx::new(unsafe {
                mem::transmute::<&Window, &'static Window>(
                    self.window.as_ref().unwrap(),
                )
            }));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.state.is_pressed() {
                    let winit::keyboard::PhysicalKey::Code(key_code) =
                        event.physical_key
                    else {
                        return;
                    };

                    let amount = 0.1;
                    match key_code {
                        KeyCode::ArrowLeft => self.rot[0] -= amount,
                        KeyCode::ArrowRight => self.rot[0] += amount,
                        KeyCode::ArrowUp => self.rot[1] -= amount,
                        KeyCode::ArrowDown => self.rot[1] += amount,
                        _ => {
                            return;
                        }
                    }
                    self.redraw();
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let [dx, dy] = [
                    self.mouse_pos[0] - position.x as f32,
                    self.mouse_pos[1] - position.y as f32,
                ];
                self.mouse_pos = [position.x as f32, position.y as f32];
                if self.drag {
                    self.cam_offset[0] -= dx;
                    self.cam_offset[1] -= dy;
                    self.redraw();
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let amount = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pt) => {
                        pt.y as f32
                    }
                };
                self.scale *= 1.01f32.powf(amount);
                self.redraw();
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if button == winit::event::MouseButton::Left {
                    self.drag = state.is_pressed();
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(gtx) = &mut self.gtx else {
                    println!("no graphics context");
                    return;
                };
                let Some(window) = &self.window else {
                    return;
                };

                gtx.draw(window, |buf| {
                    let PhysicalSize { width, height } = window.inner_size();

                    for i in 0..width {
                        for j in 0..height {
                            let x = i as f32
                                - width as f32 / 2.0
                                - self.cam_offset[0];
                            let y = j as f32
                                - height as f32 / 2.0
                                - self.cam_offset[1];

                            let mut decl =
                                2.0 * self.scale.atan2((x * x + y * y).sqrt());
                            let mut azimuth = y.atan2(x);

                            azimuth += self.rot[0];
                            decl += self.rot[1];

                            azimuth %= 2.0 * PI;
                            if azimuth < 0.0 {
                                azimuth += 2.0 * PI;
                            }

                            decl %= PI;
                            if decl < 0.0 {
                                decl += PI;
                            }

                            let decl =
                                (decl / PI * self.data_height as f32) as u32;
                            let azimuth = (azimuth / 2.0 / PI
                                * self.data_width as f32)
                                as u32;

                            let idx = (self.data_offset
                                + 3 * (decl * self.data_width + azimuth))
                                as usize;

                            let [r, g, b] =
                                GLOBE[idx..idx + 3].try_into().unwrap();

                            buf[(j * width + i) as usize] = (r as u32)
                                | ((g as u32) << 8)
                                | ((b as u32) << 16);
                        }
                    }
                })
                .unwrap();
            }
            _ => {}
        }
    }
}

fn main() {
    let bin_len = u32::from_le_bytes(GLOBE[2..6].try_into().unwrap());
    let data_offset = u32::from_le_bytes(GLOBE[10..14].try_into().unwrap());

    let width = u32::from_le_bytes(GLOBE[18..22].try_into().unwrap());
    let height = u32::from_le_bytes(GLOBE[22..26].try_into().unwrap());

    println!("Binary length: {:#x}", bin_len);
    println!("Data offset: {:#x}", data_offset);
    println!("Width: {}", width);
    println!("Height: {}", height);

    let ev_loop = EventLoop::<()>::with_user_event()
        .build()
        .expect("can't construct event loop");

    let mut app =
        App::new("tangent-proj".to_string(), data_offset, width, height);
    ev_loop.run_app(&mut app).expect("can't run app");
}
