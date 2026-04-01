#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aurora_ui::prelude::*;

#[derive(Default)]
struct FormState {
    email: String,
    password: String,
    email_error: Option<String>,
    password_error: Option<String>,
    submitted: bool,
}

/// Validate a single field and return the first error (if any).
fn validate_field(value: &str, chain: &ValidatorChain) -> Option<String> {
    chain
        .validate(value)
        .err()
        .and_then(|errors| errors.into_iter().next())
}

fn email_chain() -> ValidatorChain {
    ValidatorChain::new()
        .with(Required::new())
        .with(Email::new())
}

fn password_chain() -> ValidatorChain {
    ValidatorChain::new()
        .with(Required::new())
        .with(MinLength::new(8).message("Password must be at least 8 characters"))
}

fn main() {
    App::new()
        .title("Form Validation Example")
        .size((500, 400))
        .use_system_fonts()
        .position(WindowPosition::Center)
        .run(|window, _frame| {
            window.root(Composite::new(FormState::default(), |state, set_state| {
                let email_err = state.email_error.clone();
                let password_err = state.password_error.clone();
                let has_email_err = email_err.is_some();
                let has_password_err = password_err.is_some();
                let submitted = state.submitted;

                // Email field
                let mut email_field = Field::new("Email")
                    .required(true)
                    .description("We'll never share your email.")
                    .input(
                        TextInput::new()
                            .text(state.email.clone())
                            .placeholder("you@example.com")
                            .id(100)
                            .tab_index(1)
                            .error(has_email_err)
                            .on_change({
                                let set = set_state.clone();
                                move |text: &str| {
                                    let err = validate_field(text, &email_chain());
                                    set.set({
                                        let text = text.to_string();
                                        move |s| {
                                            s.email = text;
                                            s.email_error = err;
                                        }
                                    });
                                }
                            }),
                    )
                    .width(400.0);

                if let Some(ref err) = email_err {
                    email_field = email_field.error(err.clone());
                }

                // Password field
                let mut password_field = Field::new("Password")
                    .required(true)
                    .input(
                        TextInput::new()
                            .text(state.password.clone())
                            .placeholder("At least 8 characters")
                            .password(true)
                            .id(200)
                            .tab_index(2)
                            .error(has_password_err)
                            .on_change({
                                let set = set_state.clone();
                                move |text: &str| {
                                    let err = validate_field(text, &password_chain());
                                    set.set({
                                        let text = text.to_string();
                                        move |s| {
                                            s.password = text;
                                            s.password_error = err;
                                        }
                                    });
                                }
                            }),
                    )
                    .width(400.0);

                if let Some(ref err) = password_err {
                    password_field = password_field.error(err.clone());
                }

                // Status message
                let status = if submitted && !has_email_err && !has_password_err {
                    "Form submitted successfully!"
                } else if submitted {
                    "Please fix the errors above."
                } else {
                    ""
                };

                Box::new(
                    col!()
                        .spacing(16.0)
                        .padding(Edges::all(32.0))
                        .child(Text::new("Form Validation Demo").font_size(24.0))
                        .child(email_field)
                        .child(password_field)
                        .child(
                            button!("Submit").on_click({
                                let set = set_state.clone();
                                move |_| {
                                    set.set(|s| {
                                        s.email_error =
                                            validate_field(&s.email, &email_chain());
                                        s.password_error =
                                            validate_field(&s.password, &password_chain());
                                        s.submitted = true;
                                    });
                                }
                            }),
                        )
                        .child(Text::new(status).font_size(14.0)),
                )
            }));
        })
        .expect("Failed to run app");
}
