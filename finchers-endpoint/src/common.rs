use finchers_core::Endpoint;

#[inline(always)]
pub fn assert_output<E, T>(endpoint: E) -> E
where
    E: Endpoint<Output = T>,
{
    endpoint
}
