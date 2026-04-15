//! Pipeline error types.

use thiserror::Error;

/// Errors produced by the pipeline executor.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// The pipeline contains no effects to execute.
    #[error("pipeline contains no effects")]
    Empty,

    /// An individual effect reported a failure.
    #[error("effect '{name}' failed: {reason}")]
    EffectFailed {
        /// Name of the failing effect.
        name: &'static str,
        /// Human-readable failure description.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_error_empty_display() {
        let err = PipelineError::Empty;
        assert_eq!(err.to_string(), "pipeline contains no effects");
    }

    #[test]
    fn pipeline_error_effect_failed_display() {
        let err = PipelineError::EffectFailed {
            name: "fade_in",
            reason: "duration was zero".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "effect 'fade_in' failed: duration was zero"
        );
    }
}
