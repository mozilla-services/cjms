use crate::bigquery::client::BQClient;

pub async fn fetch_and_process_new_subscriptions(_bq: BQClient) {
    // Get all results from bigquery

    // For every result, make an entry in the subscriptions table
    // - if it doesn't exist, by flow_id
    // - append the aic_id and cj_event_value (if found in aic or aic_archive table)
    // - create an oid for reporting to cj

    // Mark status as either:
    // - subscription_to_report
    // - do_not_report (if subscription_starttime is after aic expires)
    // Add details to status_history blob

    // Move aic row to aic_archive table
}

#[cfg(test)]
mod tests {}
