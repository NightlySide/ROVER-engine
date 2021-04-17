pub mod window;
pub mod camera;
pub mod pipeline;
pub mod vertex;
pub mod texture;
pub mod uniform;
pub mod instance;
pub mod light;

use futures::executor::block_on;
use winit::{event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use pipeline::State;

pub fn run() {
    let title = env!("CARGO_PKG_NAME");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    let mut state = block_on(State::new(&window));
    let mut last_render_time = std::time::Instant::now();
    
    event_loop.run(move |event, _, control_flow|  {
        *control_flow = ControlFlow::Poll;
        match event {
            // device events
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                state.input(&window, event);
            },
            // window events
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => { 
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    },
                    _ => {}
                }
            },
            // on redraw
            Event::RedrawRequested(_) => {
                // get the time diff
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                // on each new frame we update the system
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            // set redraw
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually request it.
                window.request_redraw();
            },
            _ => {}
        }
    });
}