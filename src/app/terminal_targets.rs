use super::{api_helpers::pane_agent_status, App};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalTarget {
    pub ws_idx: usize,
    pub tab_idx: usize,
    pub pane_id: crate::layout::PaneId,
    pub terminal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TerminalTargetCandidate {
    pub terminal_id: String,
    pub pane_id: String,
    pub workspace_id: String,
    pub tab_id: String,
    pub cwd: Option<String>,
    pub agent_status: crate::api::schema::AgentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TerminalTargetError {
    NotFound {
        target: String,
    },
    Ambiguous {
        target: String,
        candidates: Vec<TerminalTargetCandidate>,
    },
}

impl App {
    pub(crate) fn resolve_terminal_target(
        &self,
        target: &str,
    ) -> Result<TerminalTarget, TerminalTargetError> {
        let terminal_matches: Vec<_> = self
            .terminal_targets()
            .into_iter()
            .filter(|candidate| candidate.terminal_id == target)
            .collect();
        if let Some(resolved) = self.single_terminal_match(target, terminal_matches)? {
            return Ok(resolved);
        }

        if let Some((ws_idx, pane_id)) = self.parse_pane_id(target) {
            if let Some(resolved) = self.terminal_target_for_pane(ws_idx, pane_id) {
                return Ok(resolved);
            }
        }

        let name_matches: Vec<_> = self
            .terminal_targets()
            .into_iter()
            .filter(|candidate| {
                self.state
                    .terminals
                    .values()
                    .find(|terminal| terminal.id.to_string() == candidate.terminal_id)
                    .is_some_and(|terminal| {
                        terminal.agent_name.as_deref() == Some(target)
                            || terminal.effective_agent_label() == Some(target)
                    })
            })
            .collect();
        if let Some(resolved) = self.single_terminal_match(target, name_matches)? {
            return Ok(resolved);
        }

        Err(TerminalTargetError::NotFound {
            target: target.to_string(),
        })
    }

    /// Resolve a pane-command target string to a concrete pane.
    ///
    /// Accepts the durable `term_<hex>` form and the existing public/raw pane id
    /// forms (`w<ws>-<n>`, `p_<raw>`, `p_<ws>_<n>`). Agent names and detected
    /// agent labels are intentionally not accepted here — they remain scoped to
    /// `agent`-prefixed commands via `resolve_terminal_target`.
    pub(crate) fn resolve_pane_target(
        &self,
        target: &str,
    ) -> Result<TerminalTarget, TerminalTargetError> {
        let terminal_matches: Vec<_> = self
            .terminal_targets()
            .into_iter()
            .filter(|candidate| candidate.terminal_id == target)
            .collect();
        if let Some(resolved) = self.single_terminal_match(target, terminal_matches)? {
            return Ok(resolved);
        }

        if let Some((ws_idx, pane_id)) = self.parse_pane_id(target) {
            if let Some(resolved) = self.terminal_target_for_pane(ws_idx, pane_id) {
                return Ok(resolved);
            }
        }

        Err(TerminalTargetError::NotFound {
            target: target.to_string(),
        })
    }

    /// Format a `TerminalTargetError` into a `pane_*`-prefixed error body for
    /// pane-command responses.
    pub(crate) fn pane_target_error_body(
        &self,
        err: TerminalTargetError,
    ) -> crate::api::schema::ErrorBody {
        match err {
            TerminalTargetError::NotFound { target } => crate::api::schema::ErrorBody {
                code: "pane_not_found".into(),
                message: format!("pane {target} not found"),
            },
            TerminalTargetError::Ambiguous { target, candidates } => {
                crate::api::schema::ErrorBody {
                    code: "pane_target_ambiguous".into(),
                    message: format!(
                        "pane target {target} is ambiguous; candidates: {}",
                        candidates
                            .into_iter()
                            .map(|candidate| format!(
                                "terminal_id={} pane_id={} workspace_id={} tab_id={} cwd={} status={:?}",
                                candidate.terminal_id,
                                candidate.pane_id,
                                candidate.workspace_id,
                                candidate.tab_id,
                                candidate.cwd.unwrap_or_else(|| "unknown".into()),
                                candidate.agent_status,
                            ))
                            .collect::<Vec<_>>()
                            .join("; ")
                    ),
                }
            }
        }
    }

    fn single_terminal_match(
        &self,
        target: &str,
        matches: Vec<TerminalTarget>,
    ) -> Result<Option<TerminalTarget>, TerminalTargetError> {
        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(TerminalTargetError::Ambiguous {
                target: target.to_string(),
                candidates: matches
                    .into_iter()
                    .filter_map(|candidate| {
                        self.terminal_target_candidate(candidate.ws_idx, candidate.pane_id)
                    })
                    .collect(),
            }),
        }
    }

    fn terminal_targets(&self) -> Vec<TerminalTarget> {
        self.state
            .workspaces
            .iter()
            .enumerate()
            .flat_map(|(ws_idx, ws)| {
                ws.tabs.iter().enumerate().flat_map(move |(tab_idx, tab)| {
                    tab.layout
                        .pane_ids()
                        .into_iter()
                        .filter_map(move |pane_id| {
                            tab.terminal_id(pane_id).map(|terminal_id| TerminalTarget {
                                ws_idx,
                                tab_idx,
                                pane_id,
                                terminal_id: terminal_id.to_string(),
                            })
                        })
                })
            })
            .collect()
    }

    fn terminal_target_for_pane(
        &self,
        ws_idx: usize,
        pane_id: crate::layout::PaneId,
    ) -> Option<TerminalTarget> {
        let ws = self.state.workspaces.get(ws_idx)?;
        let tab_idx = ws.find_tab_index_for_pane(pane_id)?;
        let terminal_id = ws.terminal_id(pane_id)?.to_string();
        Some(TerminalTarget {
            ws_idx,
            tab_idx,
            pane_id,
            terminal_id,
        })
    }

    fn terminal_target_candidate(
        &self,
        ws_idx: usize,
        pane_id: crate::layout::PaneId,
    ) -> Option<TerminalTargetCandidate> {
        let ws = self.state.workspaces.get(ws_idx)?;
        let tab_idx = ws.find_tab_index_for_pane(pane_id)?;
        let pane = ws.pane_state(pane_id)?;
        let terminal = self.state.terminals.get(&pane.attached_terminal_id)?;
        Some(TerminalTargetCandidate {
            terminal_id: terminal.id.to_string(),
            pane_id: self.public_pane_id(ws_idx, pane_id)?,
            workspace_id: self.public_workspace_id(ws_idx),
            tab_id: self.public_tab_id(ws_idx, tab_idx)?,
            cwd: ws.tabs[tab_idx]
                .cwd_for_pane(pane_id, &self.state.terminals, &self.terminal_runtimes)
                .map(|cwd| cwd.display().to_string()),
            agent_status: pane_agent_status(terminal.state, pane.seen),
        })
    }
}
