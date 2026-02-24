pub fn process_manager_sleep(alarm: &SystemTimeModel) -> (result: SleepResultModel)
    requires
        alarm.spec_wf(),
    ensures
        // The result is always one of the defined variants.
        matches!(result, SleepResultModel::Ok | SleepResultModel::TimedOut
            | SleepResultModel::Killed | SleepResultModel::GenericError { .. }),
{
    unimplemented!()
}
