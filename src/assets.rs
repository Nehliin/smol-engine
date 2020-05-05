use crate::graphics::model::Model;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use wgpu::{CommandBuffer, Device};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModelHandle {
    //TODO use a copy type instead?
    file: OsString,
}

pub struct AssetManager {
    // FxMap instead?
    asset_map: HashMap<ModelHandle, Model>,
    load_queue: VecDeque<PathBuf>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager {
            asset_map: HashMap::new(),
            load_queue: VecDeque::new(),
        }
    }

    pub fn load_model(&mut self, path: impl AsRef<Path>) -> Result<ModelHandle> {
        let path_buf = PathBuf::from(path.as_ref());
        if let Some(file_name) = path_buf.file_name() {
            let handle = ModelHandle {
                file: file_name.to_os_string(),
            };
            if self.asset_map.contains_key(&handle) {
                Ok(handle)
            } else {
                self.load_queue.push_back(path_buf);
                Ok(handle)
            }
        } else {
            Err(anyhow!(
                "The given model path isn't to an file {:?}",
                path.as_ref()
            ))
        }
    }

    pub fn get_model(&self, handle: &ModelHandle) -> Option<&Model> {
        self.asset_map.get(handle)
    }
    #[inline]
    fn clear_load_queue_impl(
        load_queue: &VecDeque<PathBuf>,
        asset_map: &mut HashMap<ModelHandle, Model>,
        device: &Device,
    ) -> Vec<CommandBuffer> {
        load_queue
            .iter()
            .map(|path_buf| {
                let (model, command_buffer) = Model::load(path_buf.as_path(), device).unwrap();
                asset_map.insert(
                    ModelHandle {
                        file: path_buf.file_name().unwrap().to_os_string(),
                    },
                    model,
                );
                command_buffer
            })
            .flatten()
            .collect::<Vec<CommandBuffer>>()
    }

    pub fn clear_load_queue(&mut self, device: &Device) -> Vec<CommandBuffer> {
        let buffers = Self::clear_load_queue_impl(&self.load_queue, &mut self.asset_map, device);
        self.load_queue.clear();
        buffers
    }
}