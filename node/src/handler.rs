use btclib::{
    network::Message,
    sha256::Hash,
    types::{Block, BlockHeader, Transaction, TransactionOutput},
    util::MerkleRoot,
};
use chrono::Utc;
use tokio::net::TcpStream;
use uuid::Uuid;

pub async fn handle_connection(mut socket: TcpStream) {
    loop {
        let message = match Message::receive_async(&mut socket).await {
            Ok(message) => message,
            Err(e) => {
                println!("Invalid message from peer: {e}, closing that connection");
                return;
            }
        };
        use btclib::network::Message::*;
        match message {
            UTXOs(_) | Template(_) | Difference(_) | TemplateValidity(_) | NodeList(_) => {
                println!("I am neither a miner nor a wallet! Goodbye");
                return;
            }
            FetchBlock(height) => {
                let blockchain = crate::BLOCKCHAIN.write().await;
                let Some(block) = blockchain.blocks().nth(height as usize).cloned()
                else {
                    return ;
                };
                let message = NewBlock(block);
                message.send_async(&mut socket).await.unwrap();
            }
            DiscoverNodes => {
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                let message = NodeList(nodes);
                message.send_async(&mut socket).await.unwrap();
            }
            AskDifference(height) => {
                let blockchain = crate::BLOCKCHAIN.write().await;
                let count = blockchain.block_height() as i32 - height as i32;
                let message = Difference(count);
                message.send_async(&mut socket).await.unwrap();
            }
            FetchUTXOs(key) => {
                println!("Received request to fetch UTXOs");
                let blockchain = crate::BLOCKCHAIN.write().await;
                let utxos = blockchain
                    .utxos()
                    .iter()
                    .filter(|(_, (_, txout))| txout.pub_key == key)
                    .map(|(_, (marked, txout))| (txout.clone(), *marked))
                    .collect::<Vec<_>>();
                let message = UTXOs(utxos);
                message.send_async(&mut socket).await.unwrap();
            }
            NewBlock(block) => {
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                println!("Received new block");
                if blockchain.add_block(block).is_err() {
                    println!("Block rejected");
                }
            }
            NewTransaction(tx) => {
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                println!("Received new transaction");
                if blockchain.add_to_mempool(tx).is_err() {
                    println!("Transaction rejected, closing connection");
                    return;
                }
            }
            ValidateTemplate(block_template) => {
                let blockchain = crate::BLOCKCHAIN.write().await;
                let status = block_template.header.prev_block_hash
                    == blockchain
                        .blocks()
                        .last()
                        .map(|last_block| last_block.hash())
                        .unwrap_or(Hash::zero());
                let message = TemplateValidity(status);
                message.send_async(&mut socket).await.unwrap();
            }
            SubmitTemplate(block) => {
                println!("Received allegedly mined template");
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                if let Err(e) = blockchain.add_block(block.clone()) {
                    println!("Block rejected: {e}, closing connection");
                    return;
                }
                blockchain.rebuild_utxos();
                println!("Block looks find, broadcasting");
                // send block to all friend nodes
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                for node in nodes {
                    if let Some(mut stream) = crate::NODES.get_mut(&node) {
                        let message = Message::NewBlock(block.clone());
                        if message.send_async(&mut *stream).await.is_err() {
                            println!("Failed to send block to {node}");
                        }
                    }
                }
            }
            SubmitTransaction(tx) => {
                println!("Submit tx");
                let mut blockchain = crate::BLOCKCHAIN.write().await;
                if let Err(e) = blockchain.add_to_mempool(tx.clone()) {
                    println!("Transaction rejected, closing connection: {e}");
                    return;
                }
                println!("Added tx to mempool");
                // send transaction to all friend nodes
                let nodes = crate::NODES
                    .iter()
                    .map(|x| x.key().clone())
                    .collect::<Vec<_>>();
                for node in nodes {
                    println!("Sending tx to friend: {}", node);
                    if let Some(mut stream) = crate::NODES.get_mut(&node) {
                        let message = Message::NewTransaction(tx.clone());
                        if message.send_async(&mut *stream).await.is_err() {
                            println!("Failed to send tx to {node}");
                        }
                    }
                }
            }
            FetchTemplate(pubkey) => {
                let blockchain = crate::BLOCKCHAIN.write().await;
                let mut txs = vec![];
                // insert transactions from mempool
                txs.extend(
                    blockchain
                        .mempool()
                        .iter()
                        .take(btclib::BLOCK_TRANSACTION_CAP as usize)
                        .map(|(_, tx)| tx)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
                // insert coinbase tx with pubkey
                txs.insert(
                    0,
                    Transaction {
                        inputs: vec![],
                        outputs: vec![TransactionOutput {
                            pub_key: pubkey,
                            unique_id: Uuid::new_v4(),
                            value: 0,
                        }],
                    },
                );
                let merkle_root = MerkleRoot::calculate(&txs);
                let mut block = Block::new(
                    BlockHeader {
                        timestamp: Utc::now(),
                        prev_block_hash: blockchain
                            .blocks()
                            .last()
                            .map(|last_block| last_block.hash())
                            .unwrap_or(Hash::zero()),
                        nonce: 0,
                        target: blockchain.target(),
                        merkle_root: merkle_root,
                    },
                    txs,
                );
                let miner_fees = match block.calcualte_block_fees(blockchain.utxos()) {
                    Ok(fees) => fees,
                    Err(e) => {
                        eprintln!("{e}");
                        return;
                    }
                };
                let reward = blockchain.calculate_block_reward();
                block.transactions[0].outputs[0].value = reward + miner_fees;

                block.header.merkle_root = MerkleRoot::calculate(&block.transactions);
                let message = Template(block);
                message.send_async(&mut socket).await.unwrap();
            }
        }
    }
}
