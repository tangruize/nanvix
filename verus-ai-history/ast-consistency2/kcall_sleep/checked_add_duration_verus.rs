pub fn checked_add_duration(now: &SystemTimeModel, timeout: &DurationModel) -> (result: Option<SystemTimeModel>)
    requires
        now.spec_wf(),
        timeout.spec_wf(),
    ensures
        spec_checked_add_succeeds(now.spec_view(), timeout.spec_view()) ==> result.is_some(),
        !spec_checked_add_succeeds(now.spec_view(), timeout.spec_view()) ==> result.is_none(),
        result.is_some() ==> result.unwrap().spec_wf(),
        result.is_some() ==> result.unwrap().spec_view() == spec_compute_alarm(now.spec_view(), timeout.spec_view()),
{
    unimplemented!()
}
