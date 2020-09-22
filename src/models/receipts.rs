use num_traits::cast::FromPrimitive;
use std::convert::TryFrom;

use bigdecimal::BigDecimal;
use serde_json::{json, Value};

use near_indexer::near_primitives::views::{ActionView, DataReceiverView};

use crate::models::enums::{ActionType, ReceiptType};
use crate::schema;
use schema::{
    receipt_action_actions, receipt_action_input_data, receipt_action_output_data, receipt_actions,
    receipt_data, receipts,
};

#[derive(Insertable, Clone)]
pub struct Receipt {
    pub receipt_id: Vec<u8>,
    pub block_height: BigDecimal,
    pub predecessor_id: String,
    pub receiver_id: String,
    pub receipt_kind: ReceiptType,
}

impl Receipt {
    pub fn from_receipt_view(
        receipt: &near_indexer::near_primitives::views::ReceiptView,
        block_height: u64,
    ) -> Self {
        Self {
            receipt_id: receipt.receipt_id.as_ref().to_vec(),
            block_height: block_height.into(),
            predecessor_id: receipt.predecessor_id.to_string(),
            receiver_id: receipt.receiver_id.to_string(),
            receipt_kind: match receipt.receipt {
                near_indexer::near_primitives::views::ReceiptEnumView::Action { .. } => {
                    ReceiptType::Action
                }
                near_indexer::near_primitives::views::ReceiptEnumView::Data { .. } => {
                    ReceiptType::Data
                }
            },
        }
    }
}

#[derive(Insertable, Clone)]
#[table_name = "receipt_data"]
pub struct ReceiptData {
    pub data_id: Vec<u8>,
    pub receipt_id: Vec<u8>,
    pub data: Option<Vec<u8>>,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for ReceiptData {
    type Error = &'static str;

    fn try_from(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, Self::Error> {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Data { data_id, data } =
            &receipt_view.receipt
        {
            Ok(Self {
                receipt_id: receipt_view.receipt_id.as_ref().to_vec(),
                data_id: data_id.as_ref().to_vec(),
                data: data.clone(),
            })
        } else {
            Err("Given ReceiptView is not Data type")
        }
    }
}

#[derive(Insertable, Clone)]
pub struct ReceiptAction {
    pub receipt_id: Vec<u8>,
    pub signer_id: String,
    pub signer_public_key: String,
    pub gas_price: BigDecimal,
}

impl TryFrom<&near_indexer::near_primitives::views::ReceiptView> for ReceiptAction {
    type Error = &'static str;

    fn try_from(
        receipt_view: &near_indexer::near_primitives::views::ReceiptView,
    ) -> Result<Self, Self::Error> {
        if let near_indexer::near_primitives::views::ReceiptEnumView::Action {
            signer_id,
            signer_public_key,
            gas_price,
            ..
        } = &receipt_view.receipt {
            Ok(
                Self {
                    receipt_id: receipt_view.receipt_id.as_ref().to_vec(),
                    signer_id: signer_id.to_string(),
                    signer_public_key: signer_public_key.to_string(),
                    gas_price: BigDecimal::from_u128(*gas_price).unwrap_or_else(||0.into()),
                }
            )
        } else {
            Err("Given ReceiptView is not Action type")
        }
    }
}

#[derive(Insertable, Clone)]
#[table_name = "receipt_action_actions"]
pub struct ReceiptActionAction {
    pub receipt_id: Vec<u8>,
    pub index: i32,
    pub action_kind: ActionType,
    pub args: serde_json::Value,
}

impl ReceiptActionAction {
    pub fn from_action_view(
        receipt_id: Vec<u8>,
        index: i32,
        action_view: &near_indexer::near_primitives::views::ActionView,
    ) -> Self {
        let (action_kind, args): (ActionType, Value) = match &action_view {
            ActionView::CreateAccount => (ActionType::CreateAccount, json!({})),
            ActionView::DeployContract { code } => {
                (ActionType::DeployContract, json!({ "code": code }))
            }
            ActionView::FunctionCall {
                method_name,
                args,
                gas,
                deposit,
            } => (
                ActionType::FunctionCall,
                json!({
                    "method_name": method_name,
                    "args": args,
                    "gas": gas,
                    "deposit": deposit.to_string(),
                }),
            ),
            ActionView::Transfer { deposit } => (
                ActionType::Transfer,
                json!({ "deposit": deposit.to_string() }),
            ),
            ActionView::Stake { stake, public_key } => (
                ActionType::Stake,
                json!({
                    "stake": stake.to_string(),
                    "public_key": public_key,
                }),
            ),
            ActionView::AddKey {
                public_key,
                access_key,
            } => (
                ActionType::AddKey,
                json!({
                    "public_key": public_key,
                    "access_key": access_key,
                }),
            ),
            ActionView::DeleteKey { public_key } => (
                ActionType::DeleteKey,
                json!({
                    "public_key": public_key,
                }),
            ),
            ActionView::DeleteAccount { beneficiary_id } => (
                ActionType::DeleteAccount,
                json!({
                    "beneficiary_id": beneficiary_id,
                }),
            ),
        };
        Self {
            receipt_id,
            index,
            args,
            action_kind,
        }
    }
}

#[derive(Insertable, Clone)]
#[table_name = "receipt_action_input_data"]
pub struct ReceiptActionInputData {
    pub receipt_id: Vec<u8>,
    pub data_id: Vec<u8>,
}

impl ReceiptActionInputData {
    pub fn from_data_id(receipt_id: Vec<u8>, data_id: Vec<u8>) -> Self {
        Self {
            receipt_id,
            data_id,
        }
    }
}

#[derive(Insertable, Clone)]
#[table_name = "receipt_action_output_data"]
pub struct ReceiptActionOutputData {
    pub receipt_id: Vec<u8>,
    pub data_id: Vec<u8>,
    pub receiver_id: String,
}

impl ReceiptActionOutputData {
    pub fn from_data_receiver(receipt_id: Vec<u8>, data_receiver: &DataReceiverView) -> Self {
        Self {
            receipt_id,
            data_id: data_receiver.data_id.as_ref().to_vec(),
            receiver_id: data_receiver.receiver_id.to_string(),
        }
    }
}
