use eframe::egui;
use paradigm_core::{wallet::Wallet, Address, transaction::Transaction};
use std::sync::{Arc, Mutex};
use anyhow::Result;

mod ui;
mod wallet_manager;
mod network_client;

use ui::WalletUI;
use wallet_manager::WalletManager;
use network_client::NetworkClient;

/// Main Paradigm Wallet Application
pub struct ParadigmWalletApp {
    wallet_manager: Arc<Mutex<WalletManager>>,
    network_client: Arc<Mutex<NetworkClient>>,
    ui: WalletUI,
    is_connected: bool,
    connection_status: String,
}

impl ParadigmWalletApp {
    pub async fn new() -> Result<Self> {
        let wallet_manager = Arc::new(Mutex::new(WalletManager::new().await?));
        let network_client = Arc::new(Mutex::new(NetworkClient::new("127.0.0.1:8080").await?));
        let ui = WalletUI::new();

        Ok(ParadigmWalletApp {
            wallet_manager,
            network_client,
            ui,
            is_connected: false,
            connection_status: "Disconnected".to_string(),
        })
    }

    async fn connect_to_network(&mut self) -> Result<()> {
        let mut client = self.network_client.lock().unwrap();
        client.connect().await?;
        self.is_connected = true;
        self.connection_status = "Connected".to_string();
        Ok(())
    }

    async fn disconnect_from_network(&mut self) -> Result<()> {
        let mut client = self.network_client.lock().unwrap();
        client.disconnect().await?;
        self.is_connected = false;
        self.connection_status = "Disconnected".to_string();
        Ok(())
    }

    async fn sync_wallet(&mut self) -> Result<()> {
        if !self.is_connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        let wallet_manager = self.wallet_manager.clone();
        let network_client = self.network_client.clone();

        // Sync wallet data with network
        let mut manager = wallet_manager.lock().unwrap();
        let mut client = network_client.lock().unwrap();
        
        if let Some(address) = manager.get_current_address() {
            // Get latest balance
            let balance = client.get_balance(&address).await?;
            manager.update_balance(balance).await?;

            // Get recent transactions
            let transactions = client.get_transactions(&address).await?;
            manager.update_transactions(transactions).await?;
        }

        Ok(())
    }
}

impl eframe::App for ParadigmWalletApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel for menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Wallet").clicked() {
                        self.ui.show_new_wallet_dialog = true;
                    }
                    if ui.button("Import Wallet").clicked() {
                        self.ui.show_import_wallet_dialog = true;
                    }
                    if ui.button("Export Wallet").clicked() {
                        self.ui.show_export_dialog = true;
                    }
                    ui.separator();
                    if ui.button("Settings").clicked() {
                        self.ui.show_settings = true;
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Network", |ui| {
                    if ui.button("Connect").clicked() && !self.is_connected {
                        // Trigger connection in async context
                        self.ui.connection_pending = true;
                    }
                    if ui.button("Disconnect").clicked() && self.is_connected {
                        // Trigger disconnection in async context
                        self.ui.disconnection_pending = true;
                    }
                    if ui.button("Sync").clicked() && self.is_connected {
                        self.ui.sync_pending = true;
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.ui.show_about = true;
                    }
                    if ui.button("Documentation").clicked() {
                        // Open documentation
                    }
                });

                // Network status
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Status: {}", self.connection_status));
                    if self.is_connected {
                        ui.colored_label(egui::Color32::GREEN, "●");
                    } else {
                        ui.colored_label(egui::Color32::RED, "●");
                    }
                });
            });
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui.show_main_content(ui, &self.wallet_manager, &self.network_client);
        });

        // Handle async operations
        if self.ui.connection_pending {
            self.ui.connection_pending = false;
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(self.connect_to_network()) {
                self.ui.error_message = Some(format!("Failed to connect: {}", e));
            }
        }

        if self.ui.disconnection_pending {
            self.ui.disconnection_pending = false;
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(self.disconnect_from_network()) {
                self.ui.error_message = Some(format!("Failed to disconnect: {}", e));
            }
        }

        if self.ui.sync_pending {
            self.ui.sync_pending = false;
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(self.sync_wallet()) {
                self.ui.error_message = Some(format!("Failed to sync: {}", e));
            }
        }

        // Show dialogs
        self.ui.show_dialogs(ctx, &self.wallet_manager);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(Arc::new(egui::IconData::default())),
        ..Default::default()
    };

    eframe::run_native(
        "Paradigm Wallet",
        options,
        Box::new(|_cc| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            Box::new(rt.block_on(ParadigmWalletApp::new()).unwrap())
        }),
    )?;

    Ok(())
}
