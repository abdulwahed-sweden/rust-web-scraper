use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

use crate::rate_limit::RateLimiter;
use crate::user_agents::get_random_user_agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtsyReviewResponse {
    #[serde(default)]
    pub reviews: Vec<ApiReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiReview {
    #[serde(rename = "review")]
    pub text: Option<String>,

    #[serde(rename = "buyer_name")]
    pub reviewer_name: Option<String>,

    pub rating: Option<i32>,

    #[serde(rename = "created_timestamp")]
    pub created_at: Option<i64>,
}

/// Extract listing ID from Etsy product URL
/// Example: https://www.etsy.com/listing/1234567890/product-name -> Some("1234567890")
pub fn extract_listing_id(url: &str) -> Option<String> {
    // Pattern: /listing/{LISTING_ID}/
    let re = Regex::new(r"/listing/(\d+)").ok()?;

    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Fetch reviews from Etsy's AJAX API for a given listing ID
/// Limits to 20 reviews maximum
pub async fn fetch_reviews(
    client: &reqwest::Client,
    listing_id: &str,
    rate_limiter: &RateLimiter,
    verbose: bool,
) -> Result<Vec<crate::etsy::Review>> {
    // Apply rate limiting
    rate_limiter.wait().await;

    // Construct the API URL
    let api_url = format!(
        "https://www.etsy.com/api/v3/ajax/bespoke/member/feedback?listing_id={}&limit=20",
        listing_id
    );

    if verbose {
        println!("      Fetching reviews from API for listing {}", listing_id);
    }

    // Make the request with retries
    let mut retries = 0;
    let max_retries = 3;

    loop {
        let user_agent = get_random_user_agent();

        let response = client
            .get(&api_url)
            .header("User-Agent", user_agent)
            .header("Accept", "application/json")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Referer", "https://www.etsy.com/")
            .header("X-Requested-With", "XMLHttpRequest")
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();

                if status.is_success() {
                    // Try to parse JSON response
                    match resp.text().await {
                        Ok(body) => {
                            // Try to parse as JSON
                            if let Ok(review_data) = serde_json::from_str::<EtsyReviewResponse>(&body) {
                                // Convert API reviews to our Review format
                                let reviews: Vec<crate::etsy::Review> = review_data
                                    .reviews
                                    .into_iter()
                                    .map(|api_review| crate::etsy::Review {
                                        text: api_review.text.unwrap_or_default(),
                                        reviewer_name: api_review.reviewer_name,
                                        rating: api_review.rating.map(|r| r.to_string()),
                                    })
                                    .collect();

                                if verbose {
                                    println!("        ✓ Fetched {} reviews", reviews.len());
                                }

                                return Ok(reviews);
                            } else {
                                // JSON parsing failed - might be HTML or different format
                                if verbose {
                                    println!("        ⚠ Could not parse review JSON (might be blocked or no reviews available)");
                                }
                                return Ok(Vec::new());
                            }
                        }
                        Err(e) => {
                            if verbose {
                                println!("        ⚠ Failed to read response body: {}", e);
                            }
                            return Ok(Vec::new());
                        }
                    }
                } else if status.as_u16() == 429 || status.as_u16() == 403 {
                    // Rate limited or forbidden
                    retries += 1;
                    if retries >= max_retries {
                        if verbose {
                            println!("        ⚠ Max retries reached (status {})", status);
                        }
                        return Ok(Vec::new());
                    }

                    let backoff_secs = 2_u64.pow(retries);
                    if verbose {
                        println!("        ⏳ Rate limited ({}), retrying in {}s...", status, backoff_secs);
                    }
                    sleep(Duration::from_secs(backoff_secs)).await;
                    continue;
                } else {
                    // Other HTTP error
                    if verbose {
                        println!("        ⚠ HTTP error: {}", status);
                    }
                    return Ok(Vec::new());
                }
            }
            Err(e) => {
                retries += 1;
                if retries >= max_retries {
                    if verbose {
                        println!("        ⚠ Request failed after {} retries: {}", max_retries, e);
                    }
                    return Ok(Vec::new());
                }

                if verbose {
                    println!("        ⏳ Request failed, retrying... ({})", e);
                }
                sleep(Duration::from_secs(2)).await;
                continue;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_listing_id() {
        let url = "https://www.etsy.com/listing/1234567890/product-name";
        assert_eq!(extract_listing_id(url), Some("1234567890".to_string()));

        let url2 = "https://www.etsy.com/se-en/listing/9876543210/another-product?ref=shop";
        assert_eq!(extract_listing_id(url2), Some("9876543210".to_string()));

        let invalid = "https://www.etsy.com/shop/store";
        assert_eq!(extract_listing_id(invalid), None);
    }
}
