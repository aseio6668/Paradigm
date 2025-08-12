use eframe::egui;
use paradigm_core::{wallet::Wallet, Address, transaction::Transaction};
use std::sync::{Arc, Mutex};
use anyhow::Result;

use crate::wallet_manager::WalletManager;
use crate::network_client::NetworkClient;

pub struct WalletUI {
    // Dialog states
    pub show_new_wallet_dialog: bool,
    pub show_import_wallet_dialog: bool,
    pub show_export_dialog: bool,
    pub show_settings: bool,
    pub show_about: bool,
    pub show_send_dialog: bool,
    pub show_receive_dialog: bool,

    // Form data
    pub import_seed_phrase: String,
    pub import_private_key: String,
    pub send_address: String,
    pub send_amount: String,
    pub send_fee: String,

    // UI state
    pub selected_tab: usize,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub connection_pending: bool,
    pub disconnection_pending: bool,
    pub sync_pending: bool,

    // Wallet display data
    pub current_balance: String,
    pub current_address: String,
    pub transaction_history: Vec<TransactionDisplay>,
}

#[derive(Clone)]
pub struct TransactionDisplay {
    pub id: String,
    pub direction: String, // "Sent" or "Received"
    pub amount: String,
    pub fee: String,
    pub address: String, // Other party's address
    pub timestamp: String,
    pub status: String,
}

impl WalletUI {
    pub fn new() -> Self {
        WalletUI {
            show_new_wallet_dialog: false,
            show_import_wallet_dialog: false,
            show_export_dialog: false,
            show_settings: false,
            show_about: false,
            show_send_dialog: false,
            show_receive_dialog: false,
            import_seed_phrase: String::new(),
            import_private_key: String::new(),
            send_address: String::new(),
            send_amount: String::new(),
            send_fee: "0.001".to_string(), // Default fee
            selected_tab: 0,
            error_message: None,
            success_message: None,
            connection_pending: false,
            disconnection_pending: false,
            sync_pending: false,
            current_balance: "0.00000000".to_string(),
            current_address: "No wallet loaded".to_string(),
            transaction_history: Vec::new(),
        }
    }

    pub fn show_main_content(
        &mut self,
        ui: &mut egui::Ui,
        wallet_manager: &Arc<Mutex<WalletManager>>,
        _network_client: &Arc<Mutex<NetworkClient>>,
    ) {
        // Update display data
        self.update_display_data(wallet_manager);

        // Show error/success messages
        self.show_messages(ui);

        // Main content with tabs
        ui.horizontal(|ui| {
            // Left sidebar with wallet overview
            ui.vertical(|ui| {
                ui.set_width(300.0);
                self.show_wallet_overview(ui);
            });

            ui.separator();

            // Main content area
            ui.vertical(|ui| {
                self.show_tabbed_content(ui, wallet_manager);
            });
        });
    }

    fn update_display_data(&mut self, wallet_manager: &Arc<Mutex<WalletManager>>) {
        if let Ok(manager) = wallet_manager.lock() {
            self.current_balance = manager.get_balance_string();
            self.current_address = manager.get_address_string();
            self.transaction_history = manager.get_transaction_history_display();
        }
    }

    fn show_messages(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.error_message.clone() {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            if ui.button("Dismiss").clicked() {
                self.error_message = None;
            }
        }

        if let Some(success) = &self.success_message.clone() {
            ui.colored_label(egui::Color32::GREEN, format!("Success: {}", success));
            if ui.button("Dismiss").clicked() {
                self.success_message = None;
            }
        }
    }

    fn show_wallet_overview(&mut self, ui: &mut egui::Ui) {
        ui.heading("Wallet Overview");
        ui.separator();

        // Balance display
        ui.label("Balance:");
        ui.heading(format!("{} PAR", self.current_balance));
        ui.add_space(10.0);

        // Address display
        ui.label("Address:");
        ui.monospace(&self.current_address);
        if ui.button("Copy Address").clicked() {
            ui.output_mut(|o| o.copied_text = self.current_address.clone());
            self.success_message = Some("Address copied to clipboard".to_string());
        }
        ui.add_space(10.0);

        // Quick actions
        ui.heading("Quick Actions");
        if ui.button("📤 Send PAR").clicked() {
            self.show_send_dialog = true;
        }
        if ui.button("📥 Receive PAR").clicked() {
            self.show_receive_dialog = true;
        }
        if ui.button("🔄 Refresh").clicked() {
            self.sync_pending = true;
        }
    }

    fn show_tabbed_content(
        &mut self,
        ui: &mut egui::Ui,
        wallet_manager: &Arc<Mutex<WalletManager>>,
    ) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_tab, 0, "Transactions");
            ui.selectable_value(&mut self.selected_tab, 1, "Send");
            ui.selectable_value(&mut self.selected_tab, 2, "Receive");
            ui.selectable_value(&mut self.selected_tab, 3, "Network");
        });

        ui.separator();

        match self.selected_tab {
            0 => self.show_transactions_tab(ui),
            1 => self.show_send_tab(ui, wallet_manager),
            2 => self.show_receive_tab(ui),
            3 => self.show_network_tab(ui),
            _ => {}
        }
    }

    fn show_transactions_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Transaction History");
        ui.separator();

        if self.transaction_history.is_empty() {
            ui.label("No transactions yet");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for tx in &self.transaction_history {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Direction indicator
                        let color = if tx.direction == "Sent" {
                            egui::Color32::RED
                        } else {
                            egui::Color32::GREEN
                        };
                        ui.colored_label(color, &tx.direction);

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.label(format!("Amount: {} PAR", tx.amount));
                            ui.label(format!("Fee: {} PAR", tx.fee));
                            ui.label(format!("Address: {}", tx.address));
                            ui.label(format!("Time: {}", tx.timestamp));
                            ui.label(format!("Status: {}", tx.status));
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
    }

    fn show_send_tab(&mut self, ui: &mut egui::Ui, wallet_manager: &Arc<Mutex<WalletManager>>) {
        ui.heading("Send PAR");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("To Address:");
            ui.text_edit_singleline(&mut self.send_address);
        });

        ui.horizontal(|ui| {
            ui.label("Amount (PAR):");
            ui.text_edit_singleline(&mut self.send_amount);
        });

        ui.horizontal(|ui| {
            ui.label("Fee (PAR):");
            ui.text_edit_singleline(&mut self.send_fee);
        });

        ui.add_space(10.0);

        if ui.button("Send Transaction").clicked() {
            if let Err(e) = self.handle_send_transaction(wallet_manager) {
                self.error_message = Some(format!("Send failed: {}", e));
            } else {
                self.success_message = Some("Transaction sent successfully".to_string());
                self.send_address.clear();
                self.send_amount.clear();
                self.send_fee = "0.001".to_string();
            }
        }

        ui.add_space(10.0);
        ui.label("Note: Transactions are processed through the ML-based consensus network.");
    }

    fn show_receive_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Receive PAR");
        ui.separator();

        ui.label("Share this address to receive PAR:");
        ui.add_space(10.0);

        // Large address display
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.monospace(&self.current_address);
            });
        });

        ui.add_space(10.0);

        if ui.button("Copy Address").clicked() {
            ui.output_mut(|o| o.copied_text = self.current_address.clone());
            self.success_message = Some("Address copied to clipboard".to_string());
        }

        // TODO: Add QR code generation
        ui.add_space(20.0);
        ui.label("QR Code: [Coming Soon]");
    }

    fn show_network_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Network Information");
        ui.separator();

        ui.label("Network: Paradigm Mainnet");
        ui.label("Consensus: ML-based Proof of Contribution");
        ui.label("Block Time: Near-instant (timestamp-based)");
        ui.label("Currency: PAR (8 decimal places)");

        ui.add_space(20.0);
        ui.heading("Network Statistics");
        // TODO: Display real network stats
        ui.label("Active Contributors: [Loading...]");
        ui.label("Tasks Completed: [Loading...]");
        ui.label("Network Difficulty: [Loading...]");
    }

    fn handle_send_transaction(&mut self, wallet_manager: &Arc<Mutex<WalletManager>>) -> Result<()> {
        // Validate inputs
        if self.send_address.is_empty() {
            return Err(anyhow::anyhow!("Address cannot be empty"));
        }

        let amount: f64 = self.send_amount.parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;
        
        let fee: f64 = self.send_fee.parse()
            .map_err(|_| anyhow::anyhow!("Invalid fee"))?;

        if amount <= 0.0 {
            return Err(anyhow::anyhow!("Amount must be positive"));
        }

        if fee <= 0.0 {
            return Err(anyhow::anyhow!("Fee must be positive"));
        }

        // Convert to smallest units (8 decimal places)
        let amount_units = (amount * 100_000_000.0) as u64;
        let fee_units = (fee * 100_000_000.0) as u64;

        // Create transaction through wallet manager
        let mut manager = wallet_manager.lock().unwrap();
        manager.send_transaction(&self.send_address, amount_units, fee_units)?;

        Ok(())
    }

    pub fn show_dialogs(&mut self, ctx: &egui::Context, wallet_manager: &Arc<Mutex<WalletManager>>) {
        // New wallet dialog
        if self.show_new_wallet_dialog {
            egui::Window::new("Create New Wallet")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Create a new Paradigm wallet?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            if let Err(e) = self.handle_new_wallet(wallet_manager) {
                                self.error_message = Some(format!("Failed to create wallet: {}", e));
                            } else {
                                self.success_message = Some("New wallet created successfully".to_string());
                            }
                            self.show_new_wallet_dialog = false;
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_new_wallet_dialog = false;
                        }
                    });
                });
        }

        // Import wallet dialog
        if self.show_import_wallet_dialog {
            egui::Window::new("Import Wallet")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Import wallet from seed phrase or private key:");
                    ui.add_space(10.0);

                    ui.label("Seed Phrase:");
                    ui.text_edit_multiline(&mut self.import_seed_phrase);

                    ui.add_space(10.0);
                    ui.label("Or Private Key:");
                    ui.text_edit_singleline(&mut self.import_private_key);

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Import").clicked() {
                            if let Err(e) = self.handle_import_wallet(wallet_manager) {
                                self.error_message = Some(format!("Failed to import wallet: {}", e));
                            } else {
                                self.success_message = Some("Wallet imported successfully".to_string());
                            }
                            self.show_import_wallet_dialog = false;
                            self.import_seed_phrase.clear();
                            self.import_private_key.clear();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_import_wallet_dialog = false;
                            self.import_seed_phrase.clear();
                            self.import_private_key.clear();
                        }
                    });
                });
        }

        // About dialog
        if self.show_about {
            egui::Window::new("About Paradigm Wallet")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Paradigm Wallet v0.1.0");
                    ui.label("A revolutionary cryptocurrency wallet");
                    ui.label("Built with Rust and egui");
                    ui.add_space(10.0);
                    ui.label("Features:");
                    ui.label("• ML-based consensus");
                    ui.label("• Near-instant transactions");
                    ui.label("• Secure key management");
                    ui.label("• Contribution rewards");
                    ui.add_space(10.0);

                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
        }
    }

    fn handle_new_wallet(&mut self, wallet_manager: &Arc<Mutex<WalletManager>>) -> Result<()> {
        let mut manager = wallet_manager.lock().unwrap();
        manager.create_new_wallet()?;
        Ok(())
    }

    fn handle_import_wallet(&mut self, wallet_manager: &Arc<Mutex<WalletManager>>) -> Result<()> {
        let mut manager = wallet_manager.lock().unwrap();
        
        if !self.import_seed_phrase.is_empty() {
            let words: Vec<String> = self.import_seed_phrase
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            manager.import_from_seed_phrase(&words)?;
        } else if !self.import_private_key.is_empty() {
            let private_key = hex::decode(&self.import_private_key)
                .map_err(|_| anyhow::anyhow!("Invalid private key format"))?;
            if private_key.len() != 32 {
                return Err(anyhow::anyhow!("Private key must be 32 bytes"));
            }
            let key_array: [u8; 32] = private_key.try_into().unwrap();
            manager.import_from_private_key(&key_array)?;
        } else {
            return Err(anyhow::anyhow!("Please provide either a seed phrase or private key"));
        }

        Ok(())
    }
}
