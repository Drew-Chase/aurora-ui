#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use aurora_ui::prelude::*;

fn main() {
    let mut initialized = false;

    App::new()
        .title("Threading Example")
        .size((500, 400))
        .resizable(false)
        .position(WindowPosition::Center)
        .use_system_fonts()
        .font_options(FontOptions::new().family("Segoe UI"))
        .run(move |window, _frame_info| {
            if !initialized {
                let spawner = window.task_spawner();
                window.root(
                    col!()
                        .spacing(16.0)
                        .padding(Edges::all(24.0))
                        .child(
                            Text::new("Threading Example")
                                .font_size(22.0)
                                .font_weight(FontWeight::Bold)
                                .height(32.0),
                        )
                        .child(spawn_demo(spawner.clone()))
                        .child(callback_demo(spawner)),
                );
                initialized = true;
            }
        })
        .expect("Failed to run app");
}

// ---------------------------------------------------------------------------
// Demo 1: StateSetter::set() from a background thread via TaskSpawner::spawn
// ---------------------------------------------------------------------------

#[derive(Default)]
struct SpawnState {
    status: String,
    loading: bool,
}

fn spawn_demo(spawner: TaskSpawner) -> impl Widget {
    Composite::new(SpawnState::default(), move |state, set_state| {
        let spawner = spawner.clone();
        let setter = set_state.clone();
        let is_loading = state.loading;

        let status_text = if state.loading {
            "Loading...".to_string()
        } else if state.status.is_empty() {
            "Click the button to start a background task".to_string()
        } else {
            state.status.clone()
        };

        Box::new(
            col!()
                .spacing(8.0)
                .child(
                    Text::new("spawn() — StateSetter from background thread")
                        .font_size(14.0)
                        .font_weight(FontWeight::SemiBold)
                        .height(20.0),
                )
                .child(Text::new(status_text).font_size(14.0).height(20.0))
                .child(
                    button!("Simulate File Load")
                        .width(470)
                        .height(36)
                        .on_click(move |_| {
                            if is_loading {
                                return;
                            }
                            setter.set(|s| {
                                s.loading = true;
                                s.status.clear();
                            });
                            let setter = setter.clone();
                            spawner.spawn(move || {
                                // Simulate heavy I/O on a background thread
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                setter.set(|s| {
                                    s.loading = false;
                                    s.status = "Loaded 1,024 lines from data.csv".to_string();
                                });
                            });
                        }),
                ),
        )
    })
}

// ---------------------------------------------------------------------------
// Demo 2: TaskSpawner::spawn_with_callback — result delivered on main thread
// ---------------------------------------------------------------------------

#[derive(Default)]
struct CallbackState {
    result: String,
    computing: bool,
}

fn callback_demo(spawner: TaskSpawner) -> impl Widget {
    Composite::new(CallbackState::default(), move |state, set_state| {
        let spawner = spawner.clone();
        let setter = set_state.clone();
        let is_computing = state.computing;

        let result_text = if state.computing {
            "Computing...".to_string()
        } else if state.result.is_empty() {
            "Click to run a computation on a background thread".to_string()
        } else {
            state.result.clone()
        };

        Box::new(
            col!()
                .spacing(8.0)
                .child(
                    Text::new("spawn_with_callback() — result on main thread")
                        .font_size(14.0)
                        .font_weight(FontWeight::SemiBold)
                        .height(20.0),
                )
                .child(Text::new(result_text).font_size(14.0).height(20.0))
                .child(
                    button!("Compute Fibonacci(40)")
                        .width(470)
                        .height(36)
                        .on_click(move |_| {
                            if is_computing {
                                return;
                            }
                            setter.set(|s| {
                                s.computing = true;
                                s.result.clear();
                            });
                            let setter = setter.clone();
                            spawner.spawn_with_callback(
                                || {
                                    // Expensive computation on background thread
                                    fn fib(n: u64) -> u64 {
                                        if n <= 1 {
                                            return n;
                                        }
                                        fib(n - 1) + fib(n - 2)
                                    }
                                    let start = std::time::Instant::now();
                                    let result = fib(40);
                                    let elapsed = start.elapsed();
                                    (result, elapsed)
                                },
                                move |(result, elapsed)| {
                                    // This closure runs on the main thread
                                    setter.set(|s| {
                                        s.computing = false;
                                        s.result = format!(
                                            "fib(40) = {} (computed in {:.1}s)",
                                            result,
                                            elapsed.as_secs_f64()
                                        );
                                    });
                                },
                            );
                        }),
                ),
        )
    })
}
