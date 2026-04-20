use crate::inscriptive::sync_manager::sync_manager::SYNC_MANAGER;

/// Prints the latest Cube batch sync height tip.
pub async fn batchtip_command(sync_manager: &SYNC_MANAGER) {
    let cube_batch_sync_height_tip = {
        let _sync_manager = sync_manager.lock().await;
        _sync_manager.cube_batch_sync_height_tip()
    };

    println!("Batch tip: #{}", cube_batch_sync_height_tip);
}
