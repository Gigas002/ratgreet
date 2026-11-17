use greetd_ipc::Request;

pub trait SafeDebug {
    fn safe_repr(&self) -> String;
}

impl SafeDebug for Request {
    fn safe_repr(&self) -> String {
        match self {
            msg @ &Request::CancelSession => format!("{:?}", msg),
            msg @ &Request::CreateSession { .. } => format!("{:?}", msg),
            &Request::PostAuthMessageResponse { .. } => "PostAuthMessageResponse".to_string(),
            msg @ &Request::StartSession { .. } => format!("{:?}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use greetd_ipc::Request;

    use super::SafeDebug;

    /// The debug log shows every request; a password must never be in it.
    #[test]
    fn responses_to_prompts_are_not_logged() {
        let request = Request::PostAuthMessageResponse {
            response: Some("hunter2".into()),
        };
        assert_eq!(request.safe_repr(), "PostAuthMessageResponse");

        let request = Request::CreateSession {
            username: "alice".into(),
        };
        assert!(request.safe_repr().contains("alice"));
        assert!(Request::CancelSession.safe_repr().contains("CancelSession"));
        let request = Request::StartSession {
            cmd: vec!["Hyprland".into()],
            env: vec![],
        };
        assert!(request.safe_repr().contains("Hyprland"));
    }
}
