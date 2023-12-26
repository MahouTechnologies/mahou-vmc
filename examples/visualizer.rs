use std::{
    env,
    error::Error,
    net::{SocketAddrV4, UdpSocket, Ipv4Addr},
    str::FromStr,
    sync::{Arc, Mutex},
};

use eframe::{egui, epaint::Vec2};
use egui_extras::{Size, TableBuilder};
use crossbeam_channel::bounded;
use glam::EulerRot;

use mahou_vmc::VmcData;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let usage = format!("Usage {} IP:PORT (IP:PORT)", &args[0]);
    if args.len() != 2 && args.len() != 3 {
        println!("{}", usage);
        std::process::exit(1);
    }   

    let addr = SocketAddrV4::from_str(&args[1])?;

    println!("Listening to {}", addr);

    let (s, r) = bounded(1);

    let data = Arc::new(Mutex::new(VmcData::default()));
    let _t = {
        let data = data.clone();
        std::thread::spawn(move || {
            let sock = UdpSocket::bind(addr.clone()).unwrap();
            loop {
                let mut buf = [0u8; 65536];
                match sock.recv_from(&mut buf) {
                    Ok((size, _)) => {
                        let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                        let _ = s.try_send(packet.clone());
                        data.lock().unwrap().update_from_packet(packet);
                    }
                    Err(e) => {
                        println!("Error receiving from socket: {}", e);
                        break;
                    }
                }
            }
        })
    };

    if args.len() == 3 {
        let receive = SocketAddrV4::from_str(&args[2])?;

        let _x = {
            std::thread::spawn(move || {
                let sock = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                let _ = sock.connect(receive.clone());
                loop {
                    let a = sock.send(&rosc::encoder::encode(&r.recv().unwrap()).unwrap());
                    if a.is_err() {
                        println!("{:?}", a);
                    }
                }
            })
        };
    }

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(Vec2 { x: 1280.0, y: 720.0 });
    eframe::run_native(
        "VMC Interposer",
        options,
        Box::new(|_cc| Box::new(MyApp { data, radians: true })),
    );
}

struct MyApp {
    data: Arc<Mutex<VmcData>>,
    radians: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let data = self.data.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("VMC Interposer");
            ui.label(format!("Status: {}", if data.status { "Tracking" } else {"Not Tracking"}));
            ui.label(format!("Time: {}", data.time));
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.radians, true, "Radians");
                ui.selectable_value(&mut self.radians, false, "Degrees");
            });

            {
                ui.heading("Root");
                let (vec, quat) = &data.root;
                ui.label(format!(
                    "Pos: {:10.6} {:10.6} {:10.6}",
                    vec.x, vec.y, vec.z,
                ));
                
                let (yaw, pitch, roll) = quat.to_euler(EulerRot::ZYX);

                if self.radians {
                    ui.label(format!(
                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                        vec.x, vec.y, vec.z, yaw, pitch, roll
                    ));
                } else {
                    ui.label(format!(
                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                        vec.x, vec.y, vec.z, yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees()
                    ));
                }
            }
        });

        egui::Window::new("Bones").default_pos([10.0, 240.0]).show(ctx, |ui| {
            TableBuilder::new(ui)
                .column(Size::initial(150.0))
                .column(Size::initial(550.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Data");
                    });
                })
                .body(|mut body| {
                    for (name, (vec, quat)) in &data.bones {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(name);
                            });
                            row.col(|ui| {
                                let (x, y, z) = quat.to_euler(EulerRot::XYZ);

                                if self.radians {
                                    ui.label(format!(
                                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                                        vec.x, vec.y, vec.z, x, y, z
                                    ));
                                } else {
                                    ui.label(format!(
                                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                                        vec.x, vec.y, vec.z, x.to_degrees(), y.to_degrees(), z.to_degrees()
                                    ));
                                }
                            });
                        });
                    }
                });
        });

        egui::Window::new("Tracker").show(ctx, |ui| {
            TableBuilder::new(ui)
                .column(Size::initial(150.0))
                .column(Size::initial(550.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Data");
                    });
                })
                .body(|mut body| {
                    for (name, (vec, quat)) in &data.tracker {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(name);
                            });
                            row.col(|ui| {
                                let (x, y, z) = quat.to_euler(EulerRot::XYZ);

                                if self.radians {
                                    ui.label(format!(
                                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                                        vec.x, vec.y, vec.z, x, y, z
                                    ));
                                } else {
                                    ui.label(format!(
                                        "Pos: {:10.6} {:10.6} {:10.6}    Rot: {:10.6} {:10.6} {:10.6}",
                                        vec.x, vec.y, vec.z, x.to_degrees(), y.to_degrees(), z.to_degrees()
                                    ));
                                }
                            });
                        });
                    }
                });
        });

        egui::Window::new("Blends").show(ctx, |ui| {
            TableBuilder::new(ui)
                .column(Size::initial(100.0))
                .column(Size::initial(100.0))
                .resizable(true)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Name");
                    });
                    header.col(|ui| {
                        ui.heading("Data");
                    });
                })
                .body(|mut body| {
                    for (name, value) in &data.blends {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(name);
                            });
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:.7}",
                                    value
                                ));
                            });
                        });
                    }
                });
        });

        ctx.request_repaint();
    }
}
