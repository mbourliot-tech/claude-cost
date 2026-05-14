use crate::types::RawUsage;
use std::collections::HashMap;

/// Prices in USD per 1M tokens, per Anthropic's official pricing page (May 2026).
/// Multipliers applied off `input` price: cache_5m = 1.25x, cache_1h = 2x, cache_read = 0.1x.
///
/// Non-Anthropic models (MiMo, DeepSeek) routed via CCR also emit cache_read tokens
/// but their cache-hit pricing does NOT follow Anthropic's 0.1x ratio. They set
/// `cache_read_per_mtok` to an explicit rate which overrides the Anthropic multiplier.
/// cache_5m/cache_1h tokens are not produced by CCR-routed models in practice.
#[derive(Debug, Clone, Copy)]
pub struct ModelPrice {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    /// Explicit cache-read rate for non-Anthropic models. When `None`, the
    /// Anthropic-style 0.1x-of-input multiplier is used.
    pub cache_read_per_mtok: Option<f64>,
}

impl ModelPrice {
    const CACHE_5M_MUL: f64 = 1.25;
    const CACHE_1H_MUL: f64 = 2.0;
    const CACHE_READ_MUL: f64 = 0.10;

    pub fn cost(&self, usage: &RawUsage) -> f64 {
        let (cache_5m, cache_1h) = usage.cache_split();
        let cache_read_rate = self
            .cache_read_per_mtok
            .unwrap_or(self.input_per_mtok * Self::CACHE_READ_MUL);
        let dollars = usage.input_tokens as f64 * self.input_per_mtok
            + cache_5m as f64 * self.input_per_mtok * Self::CACHE_5M_MUL
            + cache_1h as f64 * self.input_per_mtok * Self::CACHE_1H_MUL
            + usage.cache_read_input_tokens as f64 * cache_read_rate
            + usage.output_tokens as f64 * self.output_per_mtok;
        dollars / 1_000_000.0
    }
}

/// Resolve a Claude model id to its pricing. Returns None for unknown models
/// (caller logs and treats cost as 0 so unknown future models do not crash the app).
pub fn price_for(model: &str) -> Option<ModelPrice> {
    let m = model.to_ascii_lowercase();
    match m.as_str() {
        s if s.contains("opus-4-7") => Some(ModelPrice { input_per_mtok: 5.0,  output_per_mtok: 25.0, cache_read_per_mtok: None }),
        s if s.contains("opus-4-6") => Some(ModelPrice { input_per_mtok: 5.0,  output_per_mtok: 25.0, cache_read_per_mtok: None }),
        s if s.contains("opus-4-5") => Some(ModelPrice { input_per_mtok: 5.0,  output_per_mtok: 25.0, cache_read_per_mtok: None }),
        s if s.contains("opus-4-1") => Some(ModelPrice { input_per_mtok: 15.0, output_per_mtok: 75.0, cache_read_per_mtok: None }),
        s if s.contains("opus-4")   => Some(ModelPrice { input_per_mtok: 15.0, output_per_mtok: 75.0, cache_read_per_mtok: None }),
        s if s.contains("sonnet-4-6") => Some(ModelPrice { input_per_mtok: 3.0, output_per_mtok: 15.0, cache_read_per_mtok: None }),
        s if s.contains("sonnet-4-5") => Some(ModelPrice { input_per_mtok: 3.0, output_per_mtok: 15.0, cache_read_per_mtok: None }),
        s if s.contains("sonnet-4")   => Some(ModelPrice { input_per_mtok: 3.0, output_per_mtok: 15.0, cache_read_per_mtok: None }),
        s if s.contains("haiku-4-5")  => Some(ModelPrice { input_per_mtok: 1.0, output_per_mtok: 5.0, cache_read_per_mtok: None }),
        s if s.contains("haiku-3-5") || s.contains("haiku-3.5") => Some(ModelPrice { input_per_mtok: 0.80, output_per_mtok: 4.0, cache_read_per_mtok: None }),
        // Models routed through CCR (Claude Code Router) — not Anthropic.
        // MiMo V2.5 Pro uses Xiaomi's >256K-context tier ($2/$6) unconditionally:
        // CCR sessions routinely exceed 256K and we cannot determine per-call context
        // length from the JSONL usage block, so we pick the upper tier to avoid
        // under-billing rather than the lower tier ($1/$3) which would lie below cost.
        // cache_read_per_mtok = $0.40 (Xiaomi's 256K+ cache-hit price), ratio 0.20 of input.
        s if s.contains("mimo-v2.5-pro")     => Some(ModelPrice { input_per_mtok: 2.0,  output_per_mtok: 6.0, cache_read_per_mtok: Some(0.40) }),
        // DeepSeek V4 Pro: list price. The 75%-off promo runs until 2026-05-31; we
        // intentionally use the post-promo rate so totals stay consistent after that date.
        // cache_read_per_mtok = $0.145 (DeepSeek's list cache-hit price, ratio ~0.083 of input).
        s if s.contains("deepseek-v4-pro")   => Some(ModelPrice { input_per_mtok: 1.74, output_per_mtok: 3.48, cache_read_per_mtok: Some(0.145) }),
        // DeepSeek V4 Flash: cache_hit = $0.0028/Mtok (1/50 of input, vs Anthropic's 1/10).
        s if s.contains("deepseek-v4-flash") => Some(ModelPrice { input_per_mtok: 0.14, output_per_mtok: 0.28, cache_read_per_mtok: Some(0.0028) }),
        _ => None,
    }
}

pub fn cost_usd(usage: &RawUsage, model: &str) -> f64 {
    price_for(model).map(|p| p.cost(usage)).unwrap_or(0.0)
}

/// Returns the effective price for `model`, checking `overrides` first (exact match),
/// then falling back to the built-in `price_for` patterns.
pub fn effective_price<'a>(model: &str, overrides: &'a HashMap<String, ModelPrice>) -> Option<ModelPrice> {
    overrides.get(model).copied().or_else(|| price_for(model))
}

pub fn effective_cost(usage: &RawUsage, model: &str, overrides: &HashMap<String, ModelPrice>) -> f64 {
    effective_price(model, overrides).map(|p| p.cost(usage)).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CacheCreation, RawUsage};

    fn usage(input: u64, output: u64, c5m: u64, c1h: u64, cread: u64) -> RawUsage {
        RawUsage {
            input_tokens: input,
            output_tokens: output,
            cache_read_input_tokens: cread,
            cache_creation_input_tokens: c5m + c1h,
            cache_creation: Some(CacheCreation {
                ephemeral_5m_input_tokens: c5m,
                ephemeral_1h_input_tokens: c1h,
            }),
            service_tier: None,
            speed: None,
            server_tool_use: None,
        }
    }

    #[test]
    fn opus_4_7_pure_input_output() {
        // 1M input + 1M output on Opus 4.7 = 5 + 25 = $30
        let c = cost_usd(&usage(1_000_000, 1_000_000, 0, 0, 0), "claude-opus-4-7");
        assert!((c - 30.0).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn sonnet_4_6_cache_5m_pricing() {
        // 1M cache_5m tokens on Sonnet 4.6 = 3.0 * 1.25 = $3.75
        let c = cost_usd(&usage(0, 0, 1_000_000, 0, 0), "claude-sonnet-4-6");
        assert!((c - 3.75).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn sonnet_4_6_cache_1h_pricing() {
        // 1M cache_1h tokens on Sonnet 4.6 = 3.0 * 2.0 = $6.00
        let c = cost_usd(&usage(0, 0, 0, 1_000_000, 0), "claude-sonnet-4-6");
        assert!((c - 6.0).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn opus_4_7_cache_read_pricing() {
        // 1M cache_read tokens on Opus 4.7 = 5.0 * 0.10 = $0.50
        let c = cost_usd(&usage(0, 0, 0, 0, 1_000_000), "claude-opus-4-7");
        assert!((c - 0.50).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn haiku_4_5_combined() {
        // 100k input + 200k output + 50k cache_read on Haiku 4.5
        // = (100k * 1.0 + 50k * 0.10 + 200k * 5.0) / 1e6
        // = (100000 + 5000 + 1_000_000) / 1e6 = 1.105
        let c = cost_usd(&usage(100_000, 200_000, 0, 0, 50_000), "claude-haiku-4-5");
        assert!((c - 1.105).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn mimo_v2_5_pro_pure_input_output() {
        // 1M input + 1M output on MiMo v2.5 Pro (>256K tier) = $2.00 + $6.00 = $8.00
        let c = cost_usd(&usage(1_000_000, 1_000_000, 0, 0, 0), "mimo-v2.5-pro");
        assert!((c - 8.0).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn deepseek_v4_flash_pure_input_output() {
        // 1M input + 1M output on DeepSeek V4 Flash = $0.14 + $0.28 = $0.42
        let c = cost_usd(&usage(1_000_000, 1_000_000, 0, 0, 0), "deepseek-v4-flash");
        assert!((c - 0.42).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn deepseek_v4_pro_pure_input_output_list_price() {
        // 1M input + 1M output on DeepSeek V4 Pro (list price) = $1.74 + $3.48 = $5.22
        let c = cost_usd(&usage(1_000_000, 1_000_000, 0, 0, 0), "deepseek-v4-pro");
        assert!((c - 5.22).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn mimo_v2_5_pro_cache_read_overrides_anthropic_multiplier() {
        // 1M cache_read on MiMo V2.5 Pro = $0.40 (override), NOT $2.0 * 0.10 = $0.20
        let c = cost_usd(&usage(0, 0, 0, 0, 1_000_000), "mimo-v2.5-pro");
        assert!((c - 0.40).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn deepseek_v4_flash_cache_read_overrides_anthropic_multiplier() {
        // 1M cache_read on DeepSeek V4 Flash = $0.0028 (override), NOT $0.14 * 0.10 = $0.014
        let c = cost_usd(&usage(0, 0, 0, 0, 1_000_000), "deepseek-v4-flash");
        assert!((c - 0.0028).abs() < 1e-12, "got {c}");
    }

    #[test]
    fn deepseek_v4_pro_cache_read_overrides_anthropic_multiplier() {
        // 1M cache_read on DeepSeek V4 Pro = $0.145 (override), NOT $1.74 * 0.10 = $0.174
        let c = cost_usd(&usage(0, 0, 0, 0, 1_000_000), "deepseek-v4-pro");
        assert!((c - 0.145).abs() < 1e-9, "got {c}");
    }

    #[test]
    fn unknown_model_zero_cost() {
        let c = cost_usd(&usage(1_000_000, 1_000_000, 0, 0, 0), "claude-future-99");
        assert_eq!(c, 0.0);
    }

    #[test]
    fn synthetic_zero_cost() {
        let c = cost_usd(&usage(0, 0, 0, 0, 0), "<synthetic>");
        assert_eq!(c, 0.0);
    }

    #[test]
    fn cache_split_fallback_when_breakdown_missing() {
        // If the breakdown is absent, cache_creation_input_tokens falls to 5m bucket.
        let mut u = usage(0, 0, 1_000_000, 0, 0);
        u.cache_creation = None;
        u.cache_creation_input_tokens = 1_000_000;
        let c = cost_usd(&u, "claude-sonnet-4-6");
        assert!((c - 3.75).abs() < 1e-9, "got {c}");
    }
}
