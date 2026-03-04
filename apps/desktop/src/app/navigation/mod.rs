pub mod backstack;
pub mod breadcrumbs;
pub mod deep_link;
pub mod spec;

#[cfg(test)]
mod tests;

use spec::{load_navigation_policy, SsotPaths};

pub struct NavigationController {
    pub policy: spec::NavigationPolicySsot,
    pub backstack: backstack::Backstack,
    pub breadcrumbs: breadcrumbs::Breadcrumbs,
}

impl NavigationController {
    pub fn new() -> Result<Self, String> {
        let policy = load_navigation_policy(&SsotPaths::default())?;
        let backstack = backstack::Backstack::new(
            policy.backstack.max_depth as usize,
            policy.backstack.enabled,
        );
        let breadcrumbs = breadcrumbs::Breadcrumbs::new(policy.breadcrumbs.pattern.clone());
        Ok(Self {
            policy,
            backstack,
            breadcrumbs,
        })
    }
}
