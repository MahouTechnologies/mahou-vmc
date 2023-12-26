use glam::{Quat, Vec3};
use rosc::{OscPacket, OscType};

use std::collections::BTreeMap;

type Pos = (Vec3, Quat);

#[derive(Clone, Debug, PartialEq)]
pub struct VmcData {
    pub status: bool,
    pub time: f32,
    pub blends: BTreeMap<String, f32>,
    pub bones: BTreeMap<String, Pos>,
    pub root: Pos,
    pub tracker: BTreeMap<String, Pos>,
}

impl Default for VmcData {
    fn default() -> Self {
        Self {
            status: false,
            time: 0.0,
            blends: BTreeMap::new(),
            bones: BTreeMap::new(),
            root: (
                Vec3::new(0.0, 0.0, 0.0),
                Quat::from_xyzw(0.0, 0.0, 0.0, 0.0),
            ),
            tracker: BTreeMap::new(),
        }
    }
}

fn name_pos_from_args(args: &Vec<OscType>) -> Option<(String, Pos)> {
    let name = args[0].clone().string()?;

    let vec = Vec3::new(
        args[1].clone().float()?,
        args[2].clone().float()?,
        args[3].clone().float()?,
    );

    let quat = Quat::from_xyzw(
        args[4].clone().float()?,
        args[5].clone().float()?,
        args[6].clone().float()?,
        args[7].clone().float()?,
    );

    Some((name, (vec, quat)))
}

impl VmcData {
    pub fn update_from_packet(&mut self, packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                match msg.addr.as_str() {
                    "/VMC/Ext/Root/Pos" => {
                        if let Some((_, pos)) = name_pos_from_args(&msg.args) {
                            self.root = pos;
                        }
                    }
                    "/VMC/Ext/Bone/Pos" => {
                        if let Some((name, pos)) = name_pos_from_args(&msg.args) {
                            self.bones.insert(name, pos);
                        }
                    }
                    "/VMC/Ext/Tra/Pos" => {
                        if let Some((name, pos)) = name_pos_from_args(&msg.args) {
                            self.tracker.insert(name, pos);
                        }
                    }
                    "/VMC/Ext/Blend/Val" => {
                        (|| -> Option<()> {
                            let name = msg.args[0].clone().string()?;
                            let value = msg.args[1].clone().float()?;

                            self.blends.insert(name, value);
                            Some(())
                        })();
                    }
                    "/VMC/Ext/Blend/Apply" => {
                        // TODO: apply blends here the way Unity would
                    }
                    "/VMC/Ext/OK" => {
                        (|| -> Option<()> {
                            let value = msg.args[0].clone().int()?;

                            self.status = value == 1;
                            Some(())
                        })();
                    }
                    "/VMC/Ext/T" => {
                        let value = msg.args[0].clone().float().unwrap();

                        self.time = value;
                    }
                    _ => {
                        println!("unknown: {:?}", msg);
                    }
                }
            }
            OscPacket::Bundle(bundle) => {
                for packet in bundle.content {
                    self.update_from_packet(packet);
                }
            }
        }
    }
}
