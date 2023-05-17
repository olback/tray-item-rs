use crate::{TIError, IconSource};
use ksni::{menu::StandardItem, Handle, Icon};
use std::sync::Arc;

struct TrayItem {
    label: String,
    action: Option<Arc<dyn Fn() + Send + Sync + 'static>>
}

struct Tray {
    title: String,
    icon: IconSource,
    actions: Vec<TrayItem>
}

pub struct TrayItemLinux {
    tray: Handle<Tray>
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        self.title.clone()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn icon_name(&self) -> String {
        match &self.icon {
            IconSource::Resource(name) => name.to_string(),
            IconSource::Data{..} => String::new(),
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        match &self.icon {
            IconSource::Resource(_) => vec![],
            IconSource::Data{data, height, width} => {
                vec![Icon {
                    width: *height,
                    height: *width,
                    data: data.clone()
                }]
            },
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        self.actions.iter().map(|item| {
            let action = item.action.clone();
            if let Some(action) = action {
                StandardItem {
                    label: item.label.clone(),
                    activate: Box::new(move |_| {
                        action();
                    }),
                    ..Default::default()
                }
                .into()
            } else {
                StandardItem {
                    label: item.label.clone(),
                    enabled: false,
                    ..Default::default()
                }
                .into()
            }
        }).collect()
    }
}

impl TrayItemLinux {
    pub fn new(title: &str, icon: IconSource) -> Result<Self, TIError> {
        let svc = ksni::TrayService::new(Tray {
            title: title.to_string(),
            icon,
            actions: vec![]
        });

        let handle = svc.handle();
        svc.spawn();

        Ok(Self {
            tray: handle
        })
    }

    pub fn set_icon(&mut self, icon: IconSource) -> Result<(), TIError> {
        self.tray.update(|tray| {
            tray.icon = icon.clone()
        });

        Ok(())
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        self.tray.update(move |tray| {
            tray.actions.push(TrayItem {
                label: label.to_string(),
                action: None
            });
        });

        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() -> () + Send + Sync + 'static,
    {
        let action = Arc::new(move ||{
            cb();
        });

        self.tray.update(move |tray| {
            tray.actions.push(TrayItem {
                label: label.to_string(),
                action: Some(action.clone())
            });
        });
        Ok(())
    }
}
