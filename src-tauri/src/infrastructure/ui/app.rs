use eframe::egui;
use egui_plot::{Bar, BarChart, Plot};
use uuid::Uuid;

#[derive(PartialEq)]
enum NavModule {
    Dashboard,
    Tasks,
    Kanban,
    Reports,
    Admin,
    Settings,
}

#[derive(PartialEq)]
enum AuthState {
    Login,
    Register,
    Authenticated,
}

pub struct NexusTaskApp {
    state: crate::state::AppState,
    selected_module: NavModule,
    search_query: String,
    selected_workspace: String,
    workspaces: Vec<String>,
    tasks: Vec<crate::domain::task::Task>,
    current_user_id: Uuid,
    auth_state: AuthState,
    // Login fields
    login_username: String,
    login_password: String,
    login_error: Option<String>,
    // Register fields
    register_username: String,
    register_email: String,
    register_password: String,
    register_full_name: String,
    register_error: Option<String>,
    // JWT token
    jwt_token: Option<String>,
    // Task form fields
    show_task_form: bool,
    task_form_title: String,
    task_form_description: String,
    task_form_priority: i32,
    task_form_parent_id: Option<Uuid>,
    task_form_error: Option<String>,
}

impl NexusTaskApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, state: crate::state::AppState) -> Self {
        _cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        Self {
            state,
            selected_module: NavModule::Dashboard,
            search_query: String::new(),
            selected_workspace: String::new(),
            workspaces: vec![],
            tasks: vec![],
            current_user_id: Uuid::new_v4(),
            auth_state: AuthState::Login,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            register_username: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_full_name: String::new(),
            register_error: None,
            jwt_token: None,
            show_task_form: false,
            task_form_title: String::new(),
            task_form_description: String::new(),
            task_form_priority: 1,
            task_form_parent_id: None,
            task_form_error: None,
        }
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(15.0);
        ui.horizontal(|ui| {
            ui.add_space(5.0);
            ui.heading(egui::RichText::new("NEXUS").strong().color(egui::Color32::from_rgb(41, 121, 255)));
            ui.heading(egui::RichText::new("TASK").color(egui::Color32::WHITE));
        });
        ui.add_space(25.0);

        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing.y = 10.0;
            
            self.nav_item(ui, "🏠", "Dashboard", NavModule::Dashboard);
            self.nav_item(ui, "🌳", "Task Tree", NavModule::Tasks);
            self.nav_item(ui, "📋", "Kanban", NavModule::Kanban);
            self.nav_item(ui, "📊", "Analytics", NavModule::Reports);
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            self.nav_item(ui, "🛡️", "Admin", NavModule::Admin);
            self.nav_item(ui, "⚙️", "Settings", NavModule::Settings);
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(15.0);
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .rounding(4.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("👤");
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Sergio Admin").size(12.0).strong());
                            ui.label(egui::RichText::new("Super Admin").size(10.0).color(egui::Color32::GRAY));
                        });
                    });
                });
            ui.add_space(10.0);
        });
    }

    fn nav_item(&mut self, ui: &mut egui::Ui, icon: &str, text: &str, module: NavModule) {
        let is_selected = self.selected_module == module;
        let bg_color = if is_selected {
            egui::Color32::from_rgba_unmultiplied(41, 121, 255, 40)
        } else {
            egui::Color32::TRANSPARENT
        };
        let text_color = if is_selected {
            egui::Color32::from_rgb(41, 121, 255)
        } else {
            egui::Color32::from_gray(200)
        };

        let response = egui::Frame::none()
            .fill(bg_color)
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(egui::RichText::new(icon).size(16.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(text).size(14.0).color(text_color));
                });
            }).response;

        if response.interact(egui::Sense::click()).clicked() {
            self.selected_module = module;
        }
        
        if response.hovered() && !is_selected {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            
            // Workspace Dropdown with modern styling
            egui::Frame::none()
                .fill(egui::Color32::from_gray(40))
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    let display_text = if self.selected_workspace.is_empty() {
                        "🏢 Select Workspace".to_string()
                    } else {
                        format!("🏢 {}", self.selected_workspace)
                    };
                    egui::ComboBox::from_id_source("ws_selector")
                        .selected_text(display_text)
                        .show_ui(ui, |ui| {
                            if self.workspaces.is_empty() {
                                ui.label("No workspaces available");
                            }
                            for ws in &self.workspaces {
                                ui.selectable_value(&mut self.selected_workspace, ws.clone(), ws);
                            }
                        });
                });

            ui.add_space(30.0);
            
            // Search box
            let search_bg = egui::Color32::from_gray(35);
            egui::Frame::none()
                .fill(search_bg)
                .rounding(20.0)
                .inner_margin(egui::Margin::symmetric(15.0, 5.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.add(egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Search tasks, files, users...")
                            .frame(false)
                            .desired_width(350.0));
                    });
                });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(
                    egui::RichText::new("➕ New Task").strong().color(egui::Color32::WHITE)
                ).fill(egui::Color32::from_rgb(0, 200, 83))).clicked() {
                    // TODO: Open create task dialog
                }
                
                ui.add_space(15.0);
                if ui.button("🔔").clicked() { }
                if ui.button("📧").clicked() { }
            });
        });
    }

    fn draw_dashboard(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            ui.heading("System Overview");
            ui.add_space(20.0);

            // Stats row
            ui.horizontal(|ui| {
                self.stat_card(ui, "Active Tasks", "1,284", "📈 +12%", egui::Color32::from_rgb(41, 121, 255));
                ui.add_space(15.0);
                self.stat_card(ui, "Efficiency", "94.2%", "📉 -2.4%", egui::Color32::from_rgb(0, 200, 83));
                ui.add_space(15.0);
                self.stat_card(ui, "Overdue", "14", "⚡ Priority", egui::Color32::from_rgb(255, 82, 82));
                ui.add_space(15.0);
                self.stat_card(ui, "Team Load", "78%", "🆗 Stable", egui::Color32::from_rgb(255, 193, 7));
            });

            ui.add_space(30.0);
            
            // Middle row: Workload Chart & Recent Activity
            ui.columns(2, |cols| {
                cols[0].vertical(|ui| {
                    ui.label(egui::RichText::new("Weekly Workload Distribution").strong().size(16.0));
                    ui.add_space(10.0);
                    self.draw_workload_chart(ui);
                });
                
                cols[1].vertical(|ui| {
                    ui.label(egui::RichText::new("Recent Enterprise Activity").strong().size(16.0));
                    ui.add_space(10.0);
                    egui::Frame::none()
                        .fill(egui::Color32::from_gray(30))
                        .rounding(8.0)
                        .inner_margin(15.0)
                        .show(ui, |ui| {
                            ui.set_min_height(200.0);
                            self.activity_item(ui, "🟢", "System Backup completed successfully", "2 mins ago");
                            self.activity_item(ui, "🔵", "New workspace 'Marketing' created by Admin", "15 mins ago");
                            self.activity_item(ui, "🔴", "Critical Task 'Server Migration' is OVERDUE", "1 hour ago");
                            self.activity_item(ui, "🟡", "User 'Maria' invited to 'Engineering'", "3 hours ago");
                        });
                });
            });
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: &str, trend: &str, color: egui::Color32) {
        egui::Frame::none()
            .fill(egui::Color32::from_gray(35))
            .rounding(10.0)
            .inner_margin(18.0)
            .show(ui, |ui| {
                ui.set_min_width(200.0);
                ui.label(egui::RichText::new(title).size(13.0).color(egui::Color32::LIGHT_GRAY));
                ui.add_space(6.0);
                ui.label(egui::RichText::new(value).size(30.0).strong().color(egui::Color32::WHITE));
                ui.add_space(8.0);
                ui.label(egui::RichText::new(trend).size(11.0).color(color));
            });
    }

    fn activity_item(&self, ui: &mut egui::Ui, icon: &str, text: &str, time: &str) {
        ui.horizontal(|ui| {
            ui.label(icon);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(text).size(13.0));
                ui.label(egui::RichText::new(time).size(10.0).color(egui::Color32::GRAY));
            });
        });
        ui.add_space(8.0);
    }

    fn draw_workload_chart(&self, ui: &mut egui::Ui) {
        let chart = BarChart::new(vec![
            Bar::new(0.0, 15.0).name("Mon").fill(egui::Color32::from_rgb(41, 121, 255)),
            Bar::new(1.0, 22.0).name("Tue").fill(egui::Color32::from_rgb(41, 121, 255)),
            Bar::new(2.0, 18.0).name("Wed").fill(egui::Color32::from_rgb(41, 121, 255)),
            Bar::new(3.0, 25.0).name("Thu").fill(egui::Color32::from_rgb(41, 121, 255)),
            Bar::new(4.0, 20.0).name("Fri").fill(egui::Color32::from_rgb(41, 121, 255)),
            Bar::new(5.0, 10.0).name("Sat").fill(egui::Color32::from_rgb(76, 175, 80)),
            Bar::new(6.0, 5.0).name("Sun").fill(egui::Color32::from_rgb(76, 175, 80)),
        ]).width(0.6);

        Plot::new("workload_plot")
            .height(200.0)
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
            });
    }

    fn draw_kanban(&mut self, ui: &mut egui::Ui) {
        ui.heading("Kanban Board");
        ui.add_space(15.0);

        ui.horizontal_top(|ui| {
            self.kanban_column(ui, "BACKLOG", egui::Color32::GRAY, vec![
                ("Research Rust optimization", "P3"),
                ("Update security protocols", "P1"),
            ]);
            ui.add_space(15.0);
            self.kanban_column(ui, "IN PROGRESS", egui::Color32::from_rgb(41, 121, 255), vec![
                ("Modernize GUI components", "P2"),
                ("Axum API Integration", "P1"),
            ]);
            ui.add_space(15.0);
            self.kanban_column(ui, "REVIEW", egui::Color32::from_rgb(255, 193, 7), vec![
                ("Database Schema refactor", "P2"),
            ]);
            ui.add_space(15.0);
            self.kanban_column(ui, "DONE", egui::Color32::from_rgb(0, 200, 83), vec![
                ("Project initialization", "P1"),
                ("Setup Hexagonal structure", "P1"),
            ]);
        });
    }

    fn kanban_column(&self, ui: &mut egui::Ui, title: &str, color: egui::Color32, tasks: Vec<(&str, &str)>) {
        ui.vertical(|ui| {
            ui.set_width(220.0);
            ui.horizontal(|ui| {
                ui.colored_label(color, "⬛");
                ui.label(egui::RichText::new(title).strong().size(14.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(tasks.len().to_string()).color(egui::Color32::GRAY));
                });
            });
            ui.add_space(10.0);
            
            for (task, priority) in tasks {
                egui::Frame::none()
                    .fill(egui::Color32::from_gray(40))
                    .rounding(6.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(200.0);
                        ui.label(egui::RichText::new(task).size(13.0));
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(priority).size(10.0).color(egui::Color32::LIGHT_GRAY));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label("👤");
                            });
                        });
                    });
                ui.add_space(10.0);
            }
            
            if ui.button("➕ Add Card").clicked() { }
        });
    }

    fn draw_tasks(&mut self, ui: &mut egui::Ui) {
        ui.heading("Organizational Task Tree");
        ui.add_space(15.0);

        // Load tasks if not loaded
        if self.tasks.is_empty() && !self.selected_workspace.is_empty() {
            self.load_tasks();
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.tasks.is_empty() {
                ui.label("No tasks found. Select a workspace to load tasks.");
                ui.add_space(10.0);
                if ui.button("Create First Task").clicked() {
                    self.show_task_form = true;
                }
            } else {
                for task in &self.tasks {
                    let status = task.status.as_str();
                    let status_color = match status {
                        "done" => egui::Color32::from_rgb(76, 175, 80),
                        "in_progress" => egui::Color32::from_rgb(41, 121, 255),
                        "blocked" => egui::Color32::from_rgb(244, 67, 54),
                        _ => egui::Color32::GRAY,
                    };

                    let indent = task.depth as f32 * 25.0;

                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.set_min_height(35.0);
                                ui.add_space(indent);
                                
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.set_min_width(400.0);
                                    ui.label(egui::RichText::new(&task.title).size(14.0));
                                });

                                ui.add_space(20.0);
                                egui::Frame::none()
                                    .fill(status_color.linear_multiply(0.2))
                                    .rounding(10.0)
                                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.colored_label(status_color, egui::RichText::new(status.to_uppercase()).size(11.0).strong());
                                    });
                                
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new(format!("PRIORITY {}", task.priority)).size(11.0).color(egui::Color32::LIGHT_GRAY));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("⋮").clicked() {}
                                    if ui.button("✏️").clicked() {}
                                });
                            });
                        });
                    ui.separator();
                }
            }
        });
    }

    fn task_row(&self, ui: &mut egui::Ui, title: &str, status: &str, priority: i32, depth: i32) {
        let status_color = match status {
            "Done" => egui::Color32::from_rgb(76, 175, 80),
            "In Progress" => egui::Color32::from_rgb(41, 121, 255),
            "Blocked" => egui::Color32::from_rgb(244, 67, 54),
            _ => egui::Color32::GRAY,
        };

        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_min_height(35.0);
                    ui.add_space(depth as f32 * 25.0);
                    
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.set_min_width(400.0);
                        ui.label(egui::RichText::new(title).size(14.0));
                    });

                    ui.add_space(20.0);
                    egui::Frame::none()
                        .fill(status_color.linear_multiply(0.2))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                        .show(ui, |ui| {
                            ui.colored_label(status_color, egui::RichText::new(status).size(11.0).strong());
                        });
                    
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new(format!("PRIORITY {}", priority)).size(11.0).color(egui::Color32::LIGHT_GRAY));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⋮").clicked() {}
                        if ui.button("✏️").clicked() {}
                    });
                });
            });
        ui.separator();
    }

    fn draw_login_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(20)))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_gray(30))
                        .rounding(12.0)
                        .inner_margin(40.0)
                        .show(ui, |ui| {
                            ui.set_min_width(400.0);
                            
                            // Logo
                            ui.horizontal(|ui| {
                                ui.add_space(5.0);
                                ui.heading(egui::RichText::new("NEXUS").strong().color(egui::Color32::from_rgb(41, 121, 255)));
                                ui.heading(egui::RichText::new("TASK").color(egui::Color32::WHITE));
                            });
                            ui.add_space(30.0);
                            
                            ui.label(egui::RichText::new("Sign in to your account").size(18.0).strong());
                            ui.add_space(20.0);
                            
                            // Username field
                            ui.label("Username");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.login_username)
                                .hint_text("Enter your username")
                                .desired_width(350.0));
                            ui.add_space(15.0);
                            
                            // Password field
                            ui.label("Password");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.login_password)
                                .hint_text("Enter your password")
                                .password(true)
                                .desired_width(350.0));
                            ui.add_space(20.0);
                            
                            // Error message
                            if let Some(error) = &self.login_error {
                                ui.colored_label(egui::Color32::from_rgb(255, 82, 82), error);
                                ui.add_space(15.0);
                            }
                            
                            // Login button
                            if ui.add_sized([350.0, 40.0], egui::Button::new(
                                egui::RichText::new("Sign In").strong().color(egui::Color32::WHITE)
                            ).fill(egui::Color32::from_rgb(41, 121, 255))).clicked() {
                                self.handle_login();
                            }
                            ui.add_space(20.0);
                            
                            // Register link
                            ui.horizontal(|ui| {
                                ui.label("Don't have an account?");
                                if ui.link("Sign up").clicked() {
                                    self.auth_state = AuthState::Register;
                                    self.login_error = None;
                                }
                            });
                        });
                });
            });
    }

    fn draw_register_screen(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(20)))
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_gray(30))
                        .rounding(12.0)
                        .inner_margin(40.0)
                        .show(ui, |ui| {
                            ui.set_min_width(400.0);
                            
                            // Logo
                            ui.horizontal(|ui| {
                                ui.add_space(5.0);
                                ui.heading(egui::RichText::new("NEXUS").strong().color(egui::Color32::from_rgb(41, 121, 255)));
                                ui.heading(egui::RichText::new("TASK").color(egui::Color32::WHITE));
                            });
                            ui.add_space(30.0);
                            
                            ui.label(egui::RichText::new("Create your account").size(18.0).strong());
                            ui.add_space(20.0);
                            
                            // Username field
                            ui.label("Username");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.register_username)
                                .hint_text("Choose a username")
                                .desired_width(350.0));
                            ui.add_space(15.0);
                            
                            // Email field
                            ui.label("Email");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.register_email)
                                .hint_text("Enter your email")
                                .desired_width(350.0));
                            ui.add_space(15.0);
                            
                            // Full name field
                            ui.label("Full Name");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.register_full_name)
                                .hint_text("Enter your full name")
                                .desired_width(350.0));
                            ui.add_space(15.0);
                            
                            // Password field
                            ui.label("Password");
                            ui.add_space(5.0);
                            ui.add(egui::TextEdit::singleline(&mut self.register_password)
                                .hint_text("Choose a password")
                                .password(true)
                                .desired_width(350.0));
                            ui.add_space(20.0);
                            
                            // Error message
                            if let Some(error) = &self.register_error {
                                ui.colored_label(egui::Color32::from_rgb(255, 82, 82), error);
                                ui.add_space(15.0);
                            }
                            
                            // Register button
                            if ui.add_sized([350.0, 40.0], egui::Button::new(
                                egui::RichText::new("Sign Up").strong().color(egui::Color32::WHITE)
                            ).fill(egui::Color32::from_rgb(41, 121, 255))).clicked() {
                                self.handle_register();
                            }
                            ui.add_space(20.0);
                            
                            // Login link
                            ui.horizontal(|ui| {
                                ui.label("Already have an account?");
                                if ui.link("Sign in").clicked() {
                                    self.auth_state = AuthState::Login;
                                    self.register_error = None;
                                }
                            });
                        });
                });
            });
    }

    fn handle_login(&mut self) {
        if self.login_username.is_empty() || self.login_password.is_empty() {
            self.login_error = Some("Please fill in all fields".to_string());
            return;
        }

        // Call user service to authenticate
        let state = self.state.clone();
        let username = self.login_username.clone();
        let password = self.login_password.clone();

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                state.user_service.authenticate(&username, &password).await
            })
        }).join();

        match result {
            Ok(Ok((user, token))) => {
                self.auth_state = AuthState::Authenticated;
                self.login_error = None;
                self.current_user_id = user.id;
                self.jwt_token = Some(token);
            }
            Ok(Err(e)) => {
                self.login_error = Some(format!("Authentication failed: {}", e));
            }
            Err(e) => {
                self.login_error = Some(format!("Error: {:?}", e));
            }
        }
    }

    fn handle_register(&mut self) {
        if self.register_username.is_empty() || self.register_email.is_empty() 
            || self.register_full_name.is_empty() || self.register_password.is_empty() {
            self.register_error = Some("Please fill in all fields".to_string());
            return;
        }

        // Call user service to register
        let state = self.state.clone();
        let username = self.register_username.clone();
        let email = self.register_email.clone();
        let password = self.register_password.clone();
        let full_name = self.register_full_name.clone();

        let result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                state.user_service.register_user(username, email, password, full_name).await
            })
        }).join();

        match result {
            Ok(Ok(user)) => {
                self.auth_state = AuthState::Authenticated;
                self.register_error = None;
                self.current_user_id = user.id;
                // TODO: Store JWT token
            }
            Ok(Err(e)) => {
                self.register_error = Some(format!("Registration failed: {}", e));
            }
            Err(e) => {
                self.register_error = Some(format!("Error: {:?}", e));
            }
        }
    }

    fn load_tasks(&mut self) {
        if self.selected_workspace.is_empty() {
            return;
        }

        let state = self.state.clone();
        let workspace_name = self.selected_workspace.clone();
        let user_id = self.current_user_id;
        
        // First, find the workspace by name
        let workspaces_result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                state.workspace_service.get_user_workspaces(user_id).await
            })
        }).join();

        let workspace_id = match workspaces_result {
            Ok(Ok(workspaces)) => {
                workspaces.iter()
                    .find(|w| w.name == workspace_name)
                    .map(|w| w.id)
            }
            _ => None,
        };

        if let Some(workspace_id) = workspace_id {
            let state = self.state.clone();
            let tasks_result = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    state.task_service.get_workspace_tasks(workspace_id).await
                })
            }).join();

            if let Ok(Ok(tasks)) = tasks_result {
                self.tasks = tasks;
            }
        }
    }

    fn draw_task_form(&mut self, ctx: &egui::Context) {
        if !self.show_task_form {
            return;
        }

        egui::Window::new("Create Task")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                
                ui.label("Title");
                ui.add(egui::TextEdit::singleline(&mut self.task_form_title)
                    .hint_text("Enter task title")
                    .desired_width(350.0));
                ui.add_space(10.0);
                
                ui.label("Description");
                ui.add(egui::TextEdit::multiline(&mut self.task_form_description)
                    .hint_text("Enter task description")
                    .desired_width(350.0)
                    .desired_rows(3));
                ui.add_space(10.0);
                
                ui.label("Priority");
                ui.add(egui::Slider::new(&mut self.task_form_priority, 1..=5));
                ui.add_space(10.0);
                
                // Error message
                if let Some(error) = &self.task_form_error {
                    ui.colored_label(egui::Color32::from_rgb(255, 82, 82), error);
                    ui.add_space(10.0);
                }
                
                ui.horizontal(|ui| {
                    if ui.add_sized([150.0, 30.0], egui::Button::new(
                        egui::RichText::new("Create").strong().color(egui::Color32::WHITE)
                    ).fill(egui::Color32::from_rgb(41, 121, 255))).clicked() {
                        self.handle_create_task();
                    }
                    
                    ui.add_space(10.0);
                    
                    if ui.add_sized([150.0, 30.0], egui::Button::new("Cancel")).clicked() {
                        self.show_task_form = false;
                        self.task_form_error = None;
                        self.task_form_title.clear();
                        self.task_form_description.clear();
                        self.task_form_priority = 1;
                        self.task_form_parent_id = None;
                    }
                });
            });
    }

    fn handle_create_task(&mut self) {
        if self.task_form_title.is_empty() {
            self.task_form_error = Some("Title is required".to_string());
            return;
        }

        let state = self.state.clone();
        let title = self.task_form_title.clone();
        let description = if self.task_form_description.is_empty() {
            None
        } else {
            Some(self.task_form_description.clone())
        };
        let priority = self.task_form_priority;
        let parent_id = self.task_form_parent_id;
        let created_by = self.current_user_id;
        
        // Find workspace ID
        let workspace_name = self.selected_workspace.clone();
        let user_id = self.current_user_id;
        
        let workspaces_result = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                state.workspace_service.get_user_workspaces(user_id).await
            })
        }).join();

        let workspace_id = match workspaces_result {
            Ok(Ok(workspaces)) => {
                workspaces.iter()
                    .find(|w| w.name == workspace_name)
                    .map(|w| w.id)
            }
            _ => None,
        };

        if let Some(workspace_id) = workspace_id {
            let state = self.state.clone();
            let result = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    state.task_service.create_task(
                        title,
                        created_by,
                        workspace_id,
                        parent_id,
                        description,
                    ).await
                })
            }).join();

            match result {
                Ok(Ok(_)) => {
                    self.show_task_form = false;
                    self.task_form_error = None;
                    self.task_form_title.clear();
                    self.task_form_description.clear();
                    self.task_form_priority = 1;
                    self.task_form_parent_id = None;
                    self.load_tasks();
                }
                Ok(Err(e)) => {
                    self.task_form_error = Some(format!("Failed to create task: {}", e));
                }
                Err(e) => {
                    self.task_form_error = Some(format!("Error: {:?}", e));
                }
            }
        } else {
            self.task_form_error = Some("Please select a workspace".to_string());
        }
    }
}

impl eframe::App for NexusTaskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show login/register screen if not authenticated
        if self.auth_state != AuthState::Authenticated {
            match self.auth_state {
                AuthState::Login => self.draw_login_screen(ctx),
                AuthState::Register => self.draw_register_screen(ctx),
                _ => {}
            }
            return;
        }

        // Load workspaces on first frame after authentication
        if self.workspaces.is_empty() {
            let state = self.state.clone();
            let user_id = self.current_user_id;
            ctx.request_repaint();
            
            // Use tokio runtime to load data
            if let Err(e) = std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    match state.workspace_service.get_user_workspaces(user_id).await {
                        Ok(workspaces) => {
                            let names: Vec<String> = workspaces.iter().map(|w| w.name.clone()).collect();
                            Ok(names)
                        }
                        Err(e) => Err(format!("Failed to load workspaces: {}", e)),
                    }
                })
            }).join() {
                eprintln!("Failed to load workspaces: {:?}", e);
            }
        }

        // Load tasks when workspace is selected
        if !self.selected_workspace.is_empty() && self.tasks.is_empty() {
            self.load_tasks();
        }

        egui::SidePanel::left("side_panel")
            .resizable(false)
            .default_width(220.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(20)))
            .show(ctx, |ui| {
                self.draw_sidebar(ui);
            });

        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(25)).inner_margin(12.0))
            .show(ctx, |ui| {
                self.draw_top_bar(ui);
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_gray(25)).inner_margin(25.0))
            .show(ctx, |ui| {
                match self.selected_module {
                    NavModule::Dashboard => self.draw_dashboard(ui),
                    NavModule::Tasks => self.draw_tasks(ui),
                    NavModule::Kanban => self.draw_kanban(ui),
                    NavModule::Reports => { 
                        ui.heading("Enterprise Analytics"); 
                        ui.add_space(20.0);
                        self.draw_workload_chart(ui);
                        ui.add_space(20.0);
                        ui.label("Detailed performance reports are generated hourly.");
                    },
                    NavModule::Admin => { ui.heading("Administration"); ui.label("Control center."); },
                    NavModule::Settings => { ui.heading("Settings"); ui.label("Preferences."); },
                }
            });

        // Show task form if active
        self.draw_task_form(ctx);
    }
}
