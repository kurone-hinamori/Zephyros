// Suiko (推敲) Proofreading & Japanese Text Analysis Engine for Zephyros

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProofreadIssue {
    pub line_number: usize,
    pub category: String,
    pub severity: String,
    pub message: String,
    pub target_text: String,
    pub suggestion: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProofreadResult {
    pub issues: Vec<ProofreadIssue>,
    pub summary_prompt: String,
    pub total_characters: usize,
    pub sentence_count: usize,
    pub readability_score: u32,
}

pub struct SuikoEngine;

impl SuikoEngine {
    pub fn analyze(text: &str) -> ProofreadResult {
        let lines: Vec<&str> = text.lines().collect();
        let mut issues: Vec<ProofreadIssue> = Vec::new();

        let mut total_chars = 0;
        let mut sentences: Vec<(usize, String)> = Vec::new(); // (line_idx, sentence)

        // 1. 各行・文の抽出と個別チェック
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            total_chars += trimmed.chars().count();

            // 文への分割（。！？\n）
            let raw_sentences: Vec<&str> = trimmed
                .split_inclusive(&['。', '！', '？', '!', '?'][..])
                .filter(|s| !s.trim().is_empty())
                .collect();

            if raw_sentences.is_empty() {
                sentences.push((line_num, trimmed.to_string()));
            } else {
                for s in raw_sentences {
                    sentences.push((line_num, s.to_string()));
                }
            }

            // 行単位チェック: ら抜き言葉
            Self::check_ra_nuki(line_num, trimmed, &mut issues);

            // 行単位チェック: 冗長・重複表現
            Self::check_redundant_expressions(line_num, trimmed, &mut issues);

            // 行単位チェック: 二重否定
            Self::check_double_negatives(line_num, trimmed, &mut issues);

            // 行単位チェック: 助詞の連続・重なり
            Self::check_particle_repetition(line_num, trimmed, &mut issues);

            // 行単位チェック: ノイズ外国語・カタカナ固有名詞途切れ（「シルバー・レ（Silver Legacy）」等）
            Self::check_foreign_noise_and_truncation(line_num, trimmed, &mut issues);

            // 行単位チェック: 孤立カタカナ＋動詞ノイズ（「アしなければならない」等）
            Self::check_isolated_katakana_verbs(line_num, trimmed, &mut issues);
        }

        // 2. 文単位チェック（一文の長さ、読点密度）
        for (line_num, sentence) in &sentences {
            Self::check_sentence_length(*line_num, sentence, &mut issues);
            Self::check_punctuation_density(*line_num, sentence, &mut issues);
        }

        // 3. 全体文脈チェック（接続詞の連続、文節スタイル混在）
        Self::check_consecutive_conjunctions(&sentences, &mut issues);
        Self::check_style_consistency(text, &mut issues);

        // 4. 読みやすさ・推敲スコア計算 (100点満点)
        let sentence_count = sentences.len().max(1);
        let mut penalty = 0u32;
        for issue in &issues {
            match issue.severity.as_str() {
                "warning" => penalty += 5,
                "error" => penalty += 10,
                _ => penalty += 2,
            }
        }
        let readability_score = 100u32.saturating_sub(penalty).max(20);

        // 5. 編集AI用まとめプロンプトの生成
        let summary_prompt = Self::build_summary_prompt(&issues, total_chars, sentence_count, readability_score);

        ProofreadResult {
            issues,
            summary_prompt,
            total_characters: total_chars,
            sentence_count,
            readability_score,
        }
    }

    /// 「ら」抜き言葉のチェック
    fn check_ra_nuki(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        let ra_nuki_patterns = [
            ("見れる", "見られる"),
            ("見れて", "見られて"),
            ("食べれる", "食べられる"),
            ("食べれて", "食べられて"),
            ("来れる", "来られる"),
            ("来れて", "来られて"),
            ("起きれる", "起きられる"),
            ("出れる", "出られる"),
            ("出れて", "出られて"),
            ("着れる", "着られる"),
            ("受けれる", "受けられる"),
            ("信じれる", "信じられる"),
            ("考えれる", "考えられる"),
            ("感じれる", "感じられる"),
            ("投げれる", "投げられる"),
            ("逃げれる", "逃げられる"),
            ("助けれる", "助けられる"),
        ];

        for (pattern, replacement) in &ra_nuki_patterns {
            if text.contains(pattern) {
                issues.push(ProofreadIssue {
                    line_number: line_num,
                    category: "ら抜き言葉".to_string(),
                    severity: "warning".to_string(),
                    message: format!("「{}」は「ら抜き言葉」の可能性があります。", pattern),
                    target_text: pattern.to_string(),
                    suggestion: Some(replacement.to_string()),
                });
            }
        }
    }

    /// 冗長表現・重複表現のチェック
    fn check_redundant_expressions(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        let redundant_patterns = [
            ("まず最初に", "「最初」または「まず」に絞る"),
            ("一番最後", "「最後」に絞る"),
            ("頭痛が痛い", "「頭痛がする」または「頭が痛い」"),
            ("違和感を感じる", "「違和感を覚える」または「違和感がある」"),
            ("後で後悔", "「後悔」に絞る"),
            ("馬鹿な愚か者", "表現の重複"),
            ("予期せぬ不測", "「不測の事態」等に絞る"),
            ("各々それぞれ", "「それぞれ」または「各々」"),
            ("今の現状", "「現状」または「今」"),
            ("過半数を超える", "「過半数に達する」または「半数を超える」"),
        ];

        for (pattern, suggestion) in &redundant_patterns {
            if text.contains(pattern) {
                issues.push(ProofreadIssue {
                    line_number: line_num,
                    category: "冗長表現".to_string(),
                    severity: "warning".to_string(),
                    message: format!("「{}」は二重表現・冗長な言い回しです。", pattern),
                    target_text: pattern.to_string(),
                    suggestion: Some(suggestion.to_string()),
                });
            }
        }
    }

    /// 二重否定のチェック
    fn check_double_negatives(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        let double_negatives = [
            "ないわけではない",
            "なくもない",
            "できないわけではない",
            "知らなくはない",
            "言わないわけにはいかない",
        ];

        for pattern in &double_negatives {
            if text.contains(pattern) {
                issues.push(ProofreadIssue {
                    line_number: line_num,
                    category: "二重否定".to_string(),
                    severity: "info".to_string(),
                    message: format!("「{}」は二重否定です。遠回しな印象を与えるため肯定表現への書き換えを検討してください。", pattern),
                    target_text: pattern.to_string(),
                    suggestion: None,
                });
            }
        }
    }

    /// 助詞「の」や「が」の短期間での重複チェック
    fn check_particle_repetition(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        // 助詞「の」が1文/短文の中に3回以上出現する場合
        let no_count = text.matches('の').count();
        if no_count >= 3 && text.chars().count() <= 60 {
            issues.push(ProofreadIssue {
                line_number: line_num,
                category: "助詞重複".to_string(),
                severity: "warning".to_string(),
                message: format!("助詞「の」が短文内に{}回重複しています。文章のリズムが阻害される可能性があります。", no_count),
                target_text: text.chars().take(30).collect::<String>() + "...",
                suggestion: Some("言い換えや文の分割を検討してください。".to_string()),
            });
        }
    }

    /// 外国語ノイズ・カタカナ固有名詞途切れ（「シルバー・レ（Silver Legacy）」等）および複合英単語（heavy-duty（重厚））のチェック
    fn check_foreign_noise_and_truncation(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        // 1. カタカナ + 英語カッコ表記 (例: シルバー・レ（Silver Legacy）, アルド（Aldo）)
        for i in 0..len {
            if chars[i] == '（' || chars[i] == '(' {
                let mut kata_count = 0;
                let mut start_idx = i;
                while start_idx > 0 {
                    let c = chars[start_idx - 1];
                    if (c >= '\u{30A0}' && c <= '\u{30FF}') || c == '・' || c == 'ー' {
                        kata_count += 1;
                        start_idx -= 1;
                    } else {
                        break;
                    }
                }

                let mut eng_count = 0;
                let mut end_idx = i + 1;
                while end_idx < len && chars[end_idx] != '）' && chars[end_idx] != ')' {
                    let c = chars[end_idx];
                    if c.is_ascii_alphabetic() || c == ' ' {
                        eng_count += 1;
                    }
                    end_idx += 1;
                }

                if kata_count >= 2 && eng_count >= 2 && end_idx < len {
                    let snippet: String = chars[start_idx..=end_idx].iter().collect();
                    issues.push(ProofreadIssue {
                        line_number: line_num,
                        category: "ノイズ外国語・表記崩れ".to_string(),
                        severity: "error".to_string(),
                        message: format!("「{}」のようなカタカナ＋英語カッコ表記（または固有名詞の途切れノイズ）を検出しました。", snippet),
                        target_text: snippet,
                        suggestion: None,
                    });
                }
            }
        }

        // 2. 地の文におけるアルファベット英単語・ノイズ（例: heavy-duty（重厚）, casual 等）の包括的抽出
        let mut idx = 0;
        while idx < len {
            if chars[idx].is_ascii_alphabetic() {
                let start_e = idx;
                let mut end_e = idx;
                while end_e < len && (chars[end_e].is_ascii_alphanumeric() || chars[end_e] == '-' || chars[end_e] == '_') {
                    end_e += 1;
                }

                // 英単語の直後に （日本語） の注釈カッコが続いているか判定
                let mut paren_jap = String::new();
                let mut full_end = end_e;
                if end_e < len && (chars[end_e] == '（' || chars[end_e] == '(') {
                    let mut p_idx = end_e + 1;
                    let close_char = if chars[end_e] == '（' { '）' } else { ')' };
                    let mut buf = String::new();
                    while p_idx < len && chars[p_idx] != close_char && chars[p_idx] != '\n' {
                        buf.push(chars[p_idx]);
                        p_idx += 1;
                    }
                    if p_idx < len && chars[p_idx] == close_char && !buf.trim().is_empty() {
                        paren_jap = buf.trim().to_string();
                        full_end = p_idx + 1;
                    }
                }

                let word_target: String = chars[start_e..full_end].iter().collect();
                let base_eng: String = chars[start_e..end_e].iter().collect();

                if !paren_jap.is_empty() {
                    if !issues.iter().any(|iss| iss.target_text == word_target) {
                        issues.push(ProofreadIssue {
                            line_number: line_num,
                            category: "英単語ノイズ".to_string(),
                            severity: "warning".to_string(),
                            message: format!("日本語文脈に不要なアルファベット表記「{}」が含まれています。カッコ内の日本語「{}」へ置換統一してください。", word_target, paren_jap),
                            target_text: word_target,
                            suggestion: Some(paren_jap),
                        });
                    }
                } else if base_eng.len() >= 3 && !base_eng.chars().all(|c| c.is_ascii_uppercase()) {
                    let known_noise = ["get", "gett", "むget", "oversized", "casual", "heavy", "stylish", "heavy-duty"];
                    if known_noise.contains(&base_eng.as_str()) || base_eng.contains("get") {
                        if !issues.iter().any(|iss| iss.target_text == base_eng) {
                            issues.push(ProofreadIssue {
                                line_number: line_num,
                                category: "英単語ノイズ".to_string(),
                                severity: "warning".to_string(),
                                message: format!("日本語地の文に不要なアルファベットノイズ「{}」が混入しています。", base_eng),
                                target_text: base_eng,
                                suggestion: None,
                            });
                        }
                    }
                }

                idx = full_end;
            } else {
                idx += 1;
            }
        }

        // 3. カタカナ破片・単語 + （日本語但し書き/注釈） の検出 (例: カ（不器用）)
        for i in 0..len {
            if chars[i] == '（' || chars[i] == '(' {
                let close_char = if chars[i] == '（' { '）' } else { ')' };
                let mut start_k = i;
                let mut kata_buf = String::new();
                while start_k > 0 {
                    let c = chars[start_k - 1];
                    if (c >= '\u{30A0}' && c <= '\u{30FF}') || c == '・' || c == 'ー' {
                        kata_buf.insert(0, c);
                        start_k -= 1;
                    } else {
                        break;
                    }
                }

                if !kata_buf.is_empty() && kata_buf.chars().count() <= 3 {
                    let mut p_idx = i + 1;
                    let mut inner_jap = String::new();
                    while p_idx < len && chars[p_idx] != close_char && chars[p_idx] != '\n' {
                        let c = chars[p_idx];
                        if (c >= '\u{4E00}' && c <= '\u{9FFF}') || (c >= '\u{3040}' && c <= '\u{309F}') || (c >= '\u{30A0}' && c <= '\u{30FF}') {
                            inner_jap.push(c);
                        }
                        p_idx += 1;
                    }

                    if p_idx < len && chars[p_idx] == close_char && inner_jap.chars().count() >= 2 {
                        let full_snippet: String = chars[start_k..=p_idx].iter().collect();
                        if !issues.iter().any(|iss| iss.target_text == full_snippet) {
                            issues.push(ProofreadIssue {
                                line_number: line_num,
                                category: "注釈カッコ・表記崩れ".to_string(),
                                severity: "warning".to_string(),
                                message: format!("「{}」のようなカッコ但し書き・語句注釈による表記崩れを検出しました。カッコ内の意図された単語「{}」へ統一してください。", full_snippet, inner_jap),
                                target_text: full_snippet,
                                suggestion: Some(inner_jap),
                            });
                        }
                    }
                }
            }
        }
    }

    /// 孤立カタカナ＋動詞語尾のノイズ検出（例: 「アしなければならない」「カする」等）
    fn check_isolated_katakana_verbs(line_num: usize, text: &str, issues: &mut Vec<ProofreadIssue>) {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();

        let verb_endings = [
            "しなければ",
            "しなきゃ",
            "しなくては",
            "する",
            "した",
            "している",
            "された",
            "される",
            "すべき",
            "しよう",
            "され",
            "して",
            "しつつ",
            "でき",
            "できる",
        ];

        for i in 0..len {
            let c = chars[i];
            // カタカナ1文字判定
            if c >= '\u{30A1}' && c <= '\u{30FA}' {
                // 前の文字がカタカナでないこと
                let is_prev_kata = if i > 0 {
                    let prev_c = chars[i - 1];
                    (prev_c >= '\u{30A1}' && prev_c <= '\u{30FA}') || prev_c == 'ー' || prev_c == '・'
                } else {
                    false
                };

                // 次の文字がカタカナでないこと（単独の1文字カタカナ）
                let is_next_kata = if i + 1 < len {
                    let next_c = chars[i + 1];
                    (next_c >= '\u{30A1}' && next_c <= '\u{30FA}') || next_c == 'ー' || next_c == '・'
                } else {
                    false
                };

                if is_prev_kata || is_next_kata {
                    continue;
                }

                // 直後の文字列
                let rest: String = chars[i + 1..].iter().collect();

                for ending in &verb_endings {
                    if rest.starts_with(ending) {
                        let target_len = (1 + ending.chars().count()).min(12);
                        let snippet: String = chars[i..(i + target_len).min(len)].iter().collect();

                        if !issues.iter().any(|iss| iss.target_text == snippet) {
                            issues.push(ProofreadIssue {
                                line_number: line_num,
                                category: "文章崩れ・孤立カタカナ".to_string(),
                                severity: "error".to_string(),
                                message: format!(
                                    "「{}」のように一文字カタカナ＋動詞語尾（LLMトークン欠損・文法崩れノイズ）を検出しました。コンテクストを推定して正しい日本語表現に書き換えてください。",
                                    snippet
                                ),
                                target_text: snippet,
                                suggestion: None,
                            });
                        }
                        break;
                    }
                }
            }
        }
    }

    /// 一文の長さチェック
    fn check_sentence_length(line_num: usize, sentence: &str, issues: &mut Vec<ProofreadIssue>) {
        let len = sentence.chars().count();
        if len >= 120 {
            issues.push(ProofreadIssue {
                line_number: line_num,
                category: "一文の長さ".to_string(),
                severity: "warning".to_string(),
                message: format!("1文が{}文字と長すぎます（推奨: 80文字以下）。読者の負担になるため分割を検討してください。", len),
                target_text: sentence.chars().take(40).collect::<String>() + "...",
                suggestion: Some("2つ以上の文に分割することを推奨します。".to_string()),
            });
        }
    }

    /// 読点（、）の密度チェック
    fn check_punctuation_density(line_num: usize, sentence: &str, issues: &mut Vec<ProofreadIssue>) {
        let comma_count = sentence.matches('、').count();
        let len = sentence.chars().count();

        if comma_count >= 5 {
            issues.push(ProofreadIssue {
                line_number: line_num,
                category: "読点過多".to_string(),
                severity: "info".to_string(),
                message: format!("1文の中に読点「、」が{}個含まれています。文章を分けると読みやすくなります。", comma_count),
                target_text: sentence.chars().take(40).collect::<String>() + "...",
                suggestion: None,
            });
        } else if len >= 90 && comma_count == 0 {
            issues.push(ProofreadIssue {
                line_number: line_num,
                category: "読点不足".to_string(),
                severity: "info".to_string(),
                message: format!("{}文字の長文ですが読点「、」が一度も使われていません。", len),
                target_text: sentence.chars().take(40).collect::<String>() + "...",
                suggestion: Some("息継ぎとなる適切な位置に読点を挿入してください。".to_string()),
            });
        }
    }

    /// 接続詞の連続チェック（文頭）
    fn check_consecutive_conjunctions(sentences: &[(usize, String)], issues: &mut Vec<ProofreadIssue>) {
        let conjunctions = ["しかし", "だが", "そして", "だから", "また", "それに", "あるいは", "したがって"];
        let mut last_conj = String::new();

        for (line_num, sentence) in sentences {
            let s = sentence.trim();
            for conj in &conjunctions {
                if s.starts_with(conj) {
                    if last_conj == *conj {
                        issues.push(ProofreadIssue {
                            line_number: *line_num,
                            category: "接続詞の連用".to_string(),
                            severity: "warning".to_string(),
                            message: format!("接続詞「{}」が連続する文の頭で繰り返し使用されています。", conj),
                            target_text: conj.to_string(),
                            suggestion: Some("一方の接続詞を削るか別の表現に言い換えてください。".to_string()),
                        });
                    }
                    last_conj = conj.to_string();
                    break;
                } else {
                    last_conj.clear();
                }
            }
        }
    }

    /// 「です・ます」と「だ・である」の混在チェック（地の文）
    fn check_style_consistency(text: &str, issues: &mut Vec<ProofreadIssue>) {
        let mut desu_masu_count = 0;
        let mut da_dearu_count = 0;

        // 会話文「……」以外を対象にカウント
        let mut in_quotes = false;
        let mut prose_buf = String::new();

        for ch in text.chars() {
            if ch == '「' || ch == '『' {
                in_quotes = true;
            } else if ch == '」' || ch == '』' {
                in_quotes = false;
            } else if !in_quotes {
                prose_buf.push(ch);
            }
        }

        desu_masu_count += prose_buf.matches("です").count();
        desu_masu_count += prose_buf.matches("ます").count();
        desu_masu_count += prose_buf.matches("でした").count();

        da_dearu_count += prose_buf.matches("である").count();
        da_dearu_count += prose_buf.matches("であった").count();
        da_dearu_count += prose_buf.matches("だ。").count();
        da_dearu_count += prose_buf.matches("だった。").count();

        if desu_masu_count > 0 && da_dearu_count > 0 {
            let ratio = desu_masu_count as f64 / (desu_masu_count + da_dearu_count) as f64;
            if ratio > 0.15 && ratio < 0.85 {
                issues.push(ProofreadIssue {
                    line_number: 1,
                    category: "文末表現の混在".to_string(),
                    severity: "warning".to_string(),
                    message: format!(
                        "地の文で「です・ます」調({}回) と「だ・である」調({}回) が混在しています。",
                        desu_masu_count, da_dearu_count
                    ),
                    target_text: "地の文全体".to_string(),
                    suggestion: Some("作品のトーン（小説地の文は「だ・である」が主流）に合わせて統一してください。".to_string()),
                });
            }
        }
    }

    /// 編集AI用まとめプロンプトの構築
    fn build_summary_prompt(
        issues: &[ProofreadIssue],
        total_chars: usize,
        sentence_count: usize,
        readability_score: u32,
    ) -> String {
        if issues.is_empty() {
            return format!(
                "【Suiko推敲レポート】総文字数: {}字 / 文数: {} / 読みやすさ: {}点\n特筆すべき文章の不具合・リズムの乱れは検出されませんでした。",
                total_chars, sentence_count, readability_score
            );
        }

        let mut report = format!(
            "【Suiko自動推敲エンジンの分析レポート】\n- 総文字数: {}文字 / 総文数: {} / 推敲スコア: {}点\n- 検出された改善課題（{}件）:\n",
            total_chars, sentence_count, readability_score, issues.len()
        );

        for (idx, issue) in issues.iter().take(8).enumerate() {
            let sug_str = issue
                .suggestion
                .as_ref()
                .map(|s| format!(" → 修正案: {}", s))
                .unwrap_or_default();
            report.push_str(&format!(
                "{}. [{}行目 / {}] {}{} (該当箇所: 「{}」)\n",
                idx + 1,
                issue.line_number,
                issue.category,
                issue.message,
                sug_str,
                issue.target_text
            ));
        }

        if issues.len() > 8 {
            report.push_str(&format!("...他 {} 件の注意点があります。\n", issues.len() - 8));
        }

        report.push_str("上記Suikoの指摘事項を特に重点的に意識し、文章のリズムや文法を整えて自然で魅力的な小説本文に修正・推敲してください。");
        report
    }
}
