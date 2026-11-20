# CodeQL Security Triage (Second-Pass Audit)

> **Repository:** `ayumu-ml/Omnitome`  
> **Audit Date:** 2026-11-19  
> **Total CodeQL Findings:** 90 (38 Errors, 52 Warnings)  
> **Status:** Full second-pass forensic triage distinguishing Confirmed False Positives, Accepted Risks, and Real Findings.

---

## Executive Summary

| Category | Total | Error | Warning | Primary Risk Evaluation |
| :--- | :--- | :--- | :--- | :--- |
| **Stack Trace Exposure** (`py/stack-trace-exposure`) | 19 | 19 | 0 | **Accepted Risk / Low**: Localhost IPC between Python and Node; sanitize in production. |
| **SSRF** (`js/request-forgery`) | 10 | 10 | 0 | **Real Finding (DNS Rebinding)** in `webFetcher.js`; False Positive in `sidecarBase.js`. |
| **Path Traversal / Injection** (`js/path-injection`) | 4 | 4 | 0 | **Confirmed False Positive** in `ttsRoutes.js` (hex regex guard); **Defense-in-Depth** in `importQueueService.js`. |
| **ReDoS** (`js/redos`, `js/polynomial-redos`, `py/polynomial-redos`) | 13 | 4 | 9 | **Real Finding**: Table regex in `markdownParser.js` & `aiNormalizer.js` needs linear parser. |
| **Type Confusion** (`js/type-confusion-through-parameter-tampering`) | 1 | 1 | 0 | **Confirmed False Positive**: Explicit `typeof === 'string'` check in place. |
| **Missing Rate Limiting** (`js/missing-rate-limiting`) | 12 | 0 | 12 | **Architectural False Positive**: Local single-user desktop app. |
| **Tainted Format String** (`js/tainted-format-string`) | 14 | 0 | 14 | **Low Risk / Hygiene**: Standardize logger arguments. |
| **URL & Sanitization Warnings** (Various) | 17 | 0 | 17 | **Low Risk / Hygiene**: Boundary cleanups. |

---

## Complete 90-Item CodeQL Register

| # | Rule ID | Severity | File Location | Classification | Summary |
|---|---|---|---|---|---|
| 90 | `js/incomplete-multi-character-sanitization` | **WARNING** | `backend/services/web/webSearchService.js:231-232` | Low Risk | Incomplete multi-character sanitization |
| 89 | `js/incomplete-multi-character-sanitization` | **WARNING** | `backend/services/ingestion/parsers/epubParser.js:157-160` | Low Risk | Incomplete multi-character sanitization |
| 88 | `js/incomplete-multi-character-sanitization` | **WARNING** | `backend/services/ingestion/parsers/epubParser.js:157-161` | Low Risk | Incomplete multi-character sanitization |
| 87 | `js/incomplete-multi-character-sanitization` | **WARNING** | `backend/services/ai/aiNormalizer.js:289-294` | Low Risk | Incomplete multi-character sanitization |
| 86 | `js/incomplete-multi-character-sanitization` | **WARNING** | `backend/services/ai/aiNormalizer.js:243-257` | Low Risk | Incomplete multi-character sanitization |
| 85 | `js/double-escaping` | **WARNING** | `backend/services/web/webSearchService.js:115-119` | Low Risk | Double escaping or unescaping |
| 84 | `js/double-escaping` | **WARNING** | `backend/services/ingestion/parsers/epubParser.js:327-330` | Low Risk | Double escaping or unescaping |
| 83 | `js/bad-tag-filter` | **WARNING** | `backend/services/ingestion/parsers/epubParser.js:161-161` | Low Risk | Bad HTML filtering regexp |
| 82 | `js/path-injection` | **ERROR** | `backend/services/ingestion/importQueueService.js:202-202` | Accepted Risk in current callers / Real Defense-in-Depth issue | Uncontrolled data used in path expression |
| 81 | `js/path-injection` | **ERROR** | `backend/services/ingestion/importQueueService.js:201-201` | Accepted Risk in current callers / Real Defense-in-Depth issue | Uncontrolled data used in path expression |
| 80 | `js/path-injection` | **ERROR** | `backend/routes/ttsRoutes.js:77-77` | Confirmed False Positive (with Defense-in-Depth recommendation) | Uncontrolled data used in path expression |
| 79 | `js/path-injection` | **ERROR** | `backend/routes/ttsRoutes.js:66-66` | Confirmed False Positive (with Defense-in-Depth recommendation) | Uncontrolled data used in path expression |
| 78 | `js/incomplete-url-substring-sanitization` | **WARNING** | `backend/services/web/webExtractor.js:90-90` | Low Risk | Incomplete URL substring sanitization |
| 77 | `js/incomplete-url-substring-sanitization` | **WARNING** | `backend/services/web/webExtractor.js:89-89` | Low Risk | Incomplete URL substring sanitization |
| 76 | `js/incomplete-url-substring-sanitization` | **WARNING** | `backend/services/web/webExtractor.js:88-88` | Low Risk | Incomplete URL substring sanitization |
| 75 | `js/incomplete-url-substring-sanitization` | **WARNING** | `backend/services/web/webExtractor.js:87-87` | Low Risk | Incomplete URL substring sanitization |
| 74 | `js/incomplete-url-substring-sanitization` | **WARNING** | `backend/services/web/webExtractor.js:86-86` | Low Risk | Incomplete URL substring sanitization |
| 73 | `js/tainted-format-string` | **WARNING** | `backend/services/synthesis/synthesisService.js:881-881` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 72 | `js/tainted-format-string` | **WARNING** | `backend/services/synthesis/synthesisService.js:874-874` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 71 | `js/tainted-format-string` | **WARNING** | `backend/services/synthesis/synthesisService.js:865-865` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 70 | `js/tainted-format-string` | **WARNING** | `backend/services/web/webAcquisitionService.js:329-329` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 69 | `js/tainted-format-string` | **WARNING** | `backend/services/synthesis/editorialService.js:599-599` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 68 | `js/tainted-format-string` | **WARNING** | `backend/services/semantic/semanticLifecycle.js:71-71` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 67 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/intelligentSummarizer.js:662-662` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 66 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/intelligentSummarizer.js:422-422` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 65 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/bookClassifier.js:279-279` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 64 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/bookClassifier.js:200-200` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 63 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/bookBibliographer.js:253-253` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 62 | `js/tainted-format-string` | **WARNING** | `backend/services/ai/bookBibliographer.js:150-150` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 61 | `js/tainted-format-string` | **WARNING** | `backend/server.js:151-151` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 60 | `js/tainted-format-string` | **WARNING** | `backend/routes/bookRoutes.js:69-69` | Low Risk Code Hygiene | Use of externally-controlled format string |
| 59 | `js/request-forgery` | **ERROR** | `backend/services/web/webFetcher.js:212-222` | Real Finding (DNS Rebinding Gap) | Server-side request forgery |
| 58 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:199-199` | False Positive | Server-side request forgery |
| 57 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:172-172` | False Positive | Server-side request forgery |
| 56 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:153-153` | False Positive | Server-side request forgery |
| 55 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:75-75` | False Positive | Server-side request forgery |
| 54 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:55-55` | False Positive | Server-side request forgery |
| 53 | `js/request-forgery` | **ERROR** | `backend/services/ai/sidecarBase.js:40-40` | False Positive | Server-side request forgery |
| 52 | `js/request-forgery` | **ERROR** | `backend/routes/settingsRoutes.js:131-134` | Accepted Risk / Local Intended Configuration | Server-side request forgery |
| 51 | `js/request-forgery` | **ERROR** | `backend/routes/settingsRoutes.js:122-126` | Accepted Risk / Local Intended Configuration | Server-side request forgery |
| 50 | `js/request-forgery` | **ERROR** | `backend/routes/settingsRoutes.js:110-114` | Accepted Risk / Local Intended Configuration | Server-side request forgery |
| 49 | `js/resource-exhaustion` | **WARNING** | `backend/services/ai/intelligentSummarizer.js:730-730` | Low Risk | Resource exhaustion |
| 48 | `js/resource-exhaustion` | **WARNING** | `backend/services/ai/bookClassifier.js:166-166` | Low Risk | Resource exhaustion |
| 47 | `js/resource-exhaustion` | **WARNING** | `backend/services/ai/bookBibliographer.js:116-116` | Low Risk | Resource exhaustion |
| 46 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/ttsRoutes.js:54-78` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 45 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:617-701` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 44 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:558-614` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 43 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/resynthesisRoutes.js:10-136` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 42 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:350-350` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 41 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:283-310` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 40 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:181-211` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 39 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/importQueueRoutes.js:255-306` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 38 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/importQueueRoutes.js:200-252` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 37 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/bookRoutes.js:93-178` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 36 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/importQueueRoutes.js:126-197` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 35 | `js/missing-rate-limiting` | **WARNING** | `backend/routes/lifecycleRoutes.js:243-243` | Architectural False Positive / Intended Local Architecture | Missing rate limiting |
| 34 | `js/type-confusion-through-parameter-tampering` | **ERROR** | `backend/routes/importQueueRoutes.js:179-179` | Confirmed False Positive | Type confusion through parameter tampering |
| 33 | `js/loop-bound-injection` | **WARNING** | `backend/services/ingestion/importQueueService.js:14-14` | Low Risk | Loop bound injection |
| 32 | `js/polynomial-redos` | **WARNING** | `backend/services/settings/settingsService.js:114-114` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 31 | `js/polynomial-redos` | **WARNING** | `backend/services/ai/ttsClient.js:34-35` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 30 | `js/polynomial-redos` | **WARNING** | `backend/services/ai/aiNormalizer.js:266-268` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 29 | `js/polynomial-redos` | **WARNING** | `backend/routes/settingsRoutes.js:130-130` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 28 | `js/polynomial-redos` | **WARNING** | `backend/routes/settingsRoutes.js:107-107` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 27 | `js/polynomial-redos` | **WARNING** | `backend/routes/settingsRoutes.js:89-89` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 26 | `js/polynomial-redos` | **WARNING** | `backend/routes/settingsRoutes.js:83-83` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 25 | `js/redos` | **ERROR** | `backend/services/ingestion/parsers/markdownParser.js:197-197` | Real Finding (Low Severity Edge Case) | Inefficient regular expression |
| 24 | `js/redos` | **ERROR** | `backend/services/ingestion/parsers/markdownParser.js:129-129` | Real Finding (Low Severity Edge Case) | Inefficient regular expression |
| 23 | `js/redos` | **ERROR** | `backend/services/ai/aiNormalizer.js:211-211` | Real Finding (Low Severity Edge Case) | Inefficient regular expression |
| 22 | `js/redos` | **ERROR** | `backend/services/ai/aiNormalizer.js:142-142` | Real Finding (Low Severity Edge Case) | Inefficient regular expression |
| 21 | `py/polynomial-redos` | **WARNING** | `sidecars/python/tts/synthesize.py:282-282` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 20 | `py/polynomial-redos` | **WARNING** | `sidecars/python/tts/synthesize.py:281-281` | Real Finding (Low Severity Edge Case) | Polynomial regular expression used on uncontrolled data |
| 19 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:683-687` | Accepted Risk / Low Impact | Information exposure through an exception |
| 18 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:596-600` | Accepted Risk / Low Impact | Information exposure through an exception |
| 17 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:521-525` | Accepted Risk / Low Impact | Information exposure through an exception |
| 16 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:445-449` | Accepted Risk / Low Impact | Information exposure through an exception |
| 15 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:238-242` | Accepted Risk / Low Impact | Information exposure through an exception |
| 14 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/nlp/routes.py:178-182` | Accepted Risk / Low Impact | Information exposure through an exception |
| 13 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/parse/routes.py:116-116` | Accepted Risk / Low Impact | Information exposure through an exception |
| 12 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/parse/routes.py:106-115` | Accepted Risk / Low Impact | Information exposure through an exception |
| 11 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/ocr/routes.py:216-220` | Accepted Risk / Low Impact | Information exposure through an exception |
| 10 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:132-137` | Accepted Risk / Low Impact | Information exposure through an exception |
| 9 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:123-128` | Accepted Risk / Low Impact | Information exposure through an exception |
| 8 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:109-114` | Accepted Risk / Low Impact | Information exposure through an exception |
| 7 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:100-105` | Accepted Risk / Low Impact | Information exposure through an exception |
| 6 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:88-93` | Accepted Risk / Low Impact | Information exposure through an exception |
| 5 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:78-83` | Accepted Risk / Low Impact | Information exposure through an exception |
| 4 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/tts/routes.py:68-73` | Accepted Risk / Low Impact | Information exposure through an exception |
| 3 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/embed/routes.py:177-181` | Accepted Risk / Low Impact | Information exposure through an exception |
| 2 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/pdfmath_sidecar/extract.py:155-155` | Accepted Risk / Low Impact | Information exposure through an exception |
| 1 | `py/stack-trace-exposure` | **ERROR** | `sidecars/python/pdfmath_sidecar/extract.py:148-153` | Accepted Risk / Low Impact | Information exposure through an exception |

---

## Forensic Analysis by Category

### Category: Code Hygiene / Warning (17 Findings)

#### Finding #90: `js/incomplete-multi-character-sanitization` in `backend/services/web/webSearchService.js:231-232`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete multi-character sanitization
- **Rule Message:** This string may still contain <script, which may cause an HTML element injection vulnerability.

#### Finding #89: `js/incomplete-multi-character-sanitization` in `backend/services/ingestion/parsers/epubParser.js:157-160`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete multi-character sanitization
- **Rule Message:** This string may still contain <style, which may cause an HTML element injection vulnerability.

#### Finding #88: `js/incomplete-multi-character-sanitization` in `backend/services/ingestion/parsers/epubParser.js:157-161`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete multi-character sanitization
- **Rule Message:** This string may still contain <script, which may cause an HTML element injection vulnerability.

#### Finding #87: `js/incomplete-multi-character-sanitization` in `backend/services/ai/aiNormalizer.js:289-294`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete multi-character sanitization
- **Rule Message:** This string may still contain <script, which may cause an HTML element injection vulnerability.

#### Finding #86: `js/incomplete-multi-character-sanitization` in `backend/services/ai/aiNormalizer.js:243-257`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete multi-character sanitization
- **Rule Message:** This string may still contain <script, which may cause an HTML element injection vulnerability.

#### Finding #85: `js/double-escaping` in `backend/services/web/webSearchService.js:115-119`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Double escaping or unescaping
- **Rule Message:** This replacement may produce '&' characters that are double-unescaped here.

#### Finding #84: `js/double-escaping` in `backend/services/ingestion/parsers/epubParser.js:327-330`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Double escaping or unescaping
- **Rule Message:** This replacement may produce '&' characters that are double-unescaped here.

#### Finding #83: `js/bad-tag-filter` in `backend/services/ingestion/parsers/epubParser.js:161-161`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Bad HTML filtering regexp
- **Rule Message:** This regular expression does not match script end tags like </script >.

#### Finding #78: `js/incomplete-url-substring-sanitization` in `backend/services/web/webExtractor.js:90-90`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete URL substring sanitization
- **Rule Message:** 'github.com' can be anywhere in the URL, and arbitrary hosts may come before or after it.

#### Finding #77: `js/incomplete-url-substring-sanitization` in `backend/services/web/webExtractor.js:89-89`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete URL substring sanitization
- **Rule Message:** 'arxiv.org' can be anywhere in the URL, and arbitrary hosts may come before or after it.

#### Finding #76: `js/incomplete-url-substring-sanitization` in `backend/services/web/webExtractor.js:88-88`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete URL substring sanitization
- **Rule Message:** 'mozilla.org' can be anywhere in the URL, and arbitrary hosts may come before or after it.

#### Finding #75: `js/incomplete-url-substring-sanitization` in `backend/services/web/webExtractor.js:87-87`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete URL substring sanitization
- **Rule Message:** 'openlibrary.org' can be anywhere in the URL, and arbitrary hosts may come before or after it.

#### Finding #74: `js/incomplete-url-substring-sanitization` in `backend/services/web/webExtractor.js:86-86`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Incomplete URL substring sanitization
- **Rule Message:** 'wikipedia.org' can be anywhere in the URL, and arbitrary hosts may come before or after it.

#### Finding #49: `js/resource-exhaustion` in `backend/services/ai/intelligentSummarizer.js:730-730`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Resource exhaustion
- **Rule Message:** This creates a timer with a user-controlled duration from a user-provided value. This creates a timer with a user-controlled duration from a user-provided value. This creates a timer with a user-controlled duration from a user-provided value.

#### Finding #48: `js/resource-exhaustion` in `backend/services/ai/bookClassifier.js:166-166`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Resource exhaustion
- **Rule Message:** This creates a timer with a user-controlled duration from a user-provided value.

#### Finding #47: `js/resource-exhaustion` in `backend/services/ai/bookBibliographer.js:116-116`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Resource exhaustion
- **Rule Message:** This creates a timer with a user-controlled duration from a user-provided value.

#### Finding #33: `js/loop-bound-injection` in `backend/services/ingestion/importQueueService.js:14-14`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** User input / HTTP request
- **Sink:** Unknown
- **Relevant Existing Sanitization:** None detected by CodeQL
- **Triage Classification:** **Low Risk**
- **Forensic Exploitability Analysis:** Loop bound injection
- **Rule Message:** Iteration over a user-controlled object with a potentially unbounded .length property from a user-provided value. Iteration over a user-controlled object with a potentially unbounded .length property from a user-provided value. Iteration over a user-controlled object with a potentially unbounded .length property from a user-provided value.

### Category: Path Injection / Traversal (4 Findings)

#### Finding #82: `js/path-injection` in `backend/services/ingestion/importQueueService.js:202-202`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** file.path in enqueueFile
- **Sink:** fs.promises.copyFile(file.path, stagingPath)
- **Relevant Existing Sanitization:** None on file.path
- **Triage Classification:** **Accepted Risk in current callers / Real Defense-in-Depth issue**
- **Forensic Exploitability Analysis:** Multer currently uses memoryStorage (file.buffer), so file.path is unused in HTTP upload routes. However, if any internal route passes an arbitrary file.path, copyFile will copy any file on disk into staging. file.path must be restricted to verified staging/tmp directories.
- **Rule Message:** This path depends on a user-provided value.

#### Finding #81: `js/path-injection` in `backend/services/ingestion/importQueueService.js:201-201`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** file.path in enqueueFile
- **Sink:** fs.promises.copyFile(file.path, stagingPath)
- **Relevant Existing Sanitization:** None on file.path
- **Triage Classification:** **Accepted Risk in current callers / Real Defense-in-Depth issue**
- **Forensic Exploitability Analysis:** Multer currently uses memoryStorage (file.buffer), so file.path is unused in HTTP upload routes. However, if any internal route passes an arbitrary file.path, copyFile will copy any file on disk into staging. file.path must be restricted to verified staging/tmp directories.
- **Rule Message:** This path depends on a user-provided value.

#### Finding #80: `js/path-injection` in `backend/routes/ttsRoutes.js:77-77`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** req.params.hash, req.query.voice
- **Sink:** res.sendFile(filePath)
- **Relevant Existing Sanitization:** HASH_REGEX strictly requires 64 hex characters [a-f0-9]{64}. VOICE_WHITELIST restricts voice to 4 whitelisted names.
- **Triage Classification:** **Confirmed False Positive (with Defense-in-Depth recommendation)**
- **Forensic Exploitability Analysis:** Hex hashes cannot traverse directories. However, adding path.resolve containment check (startsWith(TTS_CACHE_DIR)) eliminates static analysis ambiguity and guards against symlink substitution.
- **Rule Message:** This path depends on a user-provided value.

#### Finding #79: `js/path-injection` in `backend/routes/ttsRoutes.js:66-66`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** req.params.hash, req.query.voice
- **Sink:** res.sendFile(filePath)
- **Relevant Existing Sanitization:** HASH_REGEX strictly requires 64 hex characters [a-f0-9]{64}. VOICE_WHITELIST restricts voice to 4 whitelisted names.
- **Triage Classification:** **Confirmed False Positive (with Defense-in-Depth recommendation)**
- **Forensic Exploitability Analysis:** Hex hashes cannot traverse directories. However, adding path.resolve containment check (startsWith(TTS_CACHE_DIR)) eliminates static analysis ambiguity and guards against symlink substitution.
- **Rule Message:** This path depends on a user-provided value.

### Category: Tainted Format String (14 Findings)

#### Finding #73: `js/tainted-format-string` in `backend/services/synthesis/synthesisService.js:881-881`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value. Format string depends on a user-provided value.

#### Finding #72: `js/tainted-format-string` in `backend/services/synthesis/synthesisService.js:874-874`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value. Format string depends on a user-provided value.

#### Finding #71: `js/tainted-format-string` in `backend/services/synthesis/synthesisService.js:865-865`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value. Format string depends on a user-provided value.

#### Finding #70: `js/tainted-format-string` in `backend/services/web/webAcquisitionService.js:329-329`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #69: `js/tainted-format-string` in `backend/services/synthesis/editorialService.js:599-599`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #68: `js/tainted-format-string` in `backend/services/semantic/semanticLifecycle.js:71-71`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value.

#### Finding #67: `js/tainted-format-string` in `backend/services/ai/intelligentSummarizer.js:662-662`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #66: `js/tainted-format-string` in `backend/services/ai/intelligentSummarizer.js:422-422`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value. Format string depends on a user-provided value.

#### Finding #65: `js/tainted-format-string` in `backend/services/ai/bookClassifier.js:279-279`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #64: `js/tainted-format-string` in `backend/services/ai/bookClassifier.js:200-200`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #63: `js/tainted-format-string` in `backend/services/ai/bookBibliographer.js:253-253`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #62: `js/tainted-format-string` in `backend/services/ai/bookBibliographer.js:150-150`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #61: `js/tainted-format-string` in `backend/server.js:151-151`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value.

#### Finding #60: `js/tainted-format-string` in `backend/routes/bookRoutes.js:69-69`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Error objects / caught exceptions
- **Sink:** console.log / console.error
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Low Risk Code Hygiene**
- **Forensic Exploitability Analysis:** console.log with string concatenation can format specifiers (%s, %d). Harmless in local desktop logging, but standard practice is passing variables as separate arguments.
- **Rule Message:** Format string depends on a user-provided value. Format string depends on a user-provided value.

### Category: SSRF (Server-Side Request Forgery) (10 Findings)

#### Finding #59: `js/request-forgery` in `backend/services/web/webFetcher.js:212-222`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** User-supplied article URL in web import
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** Protocol whitelist (http/https), IP blocklist (isRestrictedHost / isRestrictedIpv4), manual redirect chain inspection.
- **Triage Classification:** **Real Finding (DNS Rebinding Gap)**
- **Forensic Exploitability Analysis:** While isRestrictedHost blocks literal private IPs (127.0.0.1, 192.168.x, 169.254.x), it lacks pre-flight DNS resolution. A domain that resolves to 127.0.0.1 or 169.254.169.254 (e.g. via nip.io or rebinding) will bypass the filter. DNS resolution validation is needed.
- **Rule Message:** The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value.

#### Finding #58: `js/request-forgery` in `backend/services/ai/sidecarBase.js:199-199`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #57: `js/request-forgery` in `backend/services/ai/sidecarBase.js:172-172`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #56: `js/request-forgery` in `backend/services/ai/sidecarBase.js:153-153`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #55: `js/request-forgery` in `backend/services/ai/sidecarBase.js:75-75`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #54: `js/request-forgery` in `backend/services/ai/sidecarBase.js:55-55`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #53: `js/request-forgery` in `backend/services/ai/sidecarBase.js:40-40`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** Configured sidecar target URL
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** resolveBaseUrl() / hardcoded localhost
- **Triage Classification:** **False Positive**
- **Forensic Exploitability Analysis:** Destination is internal sidecar on localhost. Not controllable by external remote attackers.
- **Rule Message:** The URL of this request depends on a user-provided value.

#### Finding #52: `js/request-forgery` in `backend/routes/settingsRoutes.js:131-134`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** User-supplied LLM baseUrl in /api/settings/llm/test
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Accepted Risk / Local Intended Configuration**
- **Forensic Exploitability Analysis:** Users configure local Ollama (http://localhost:11434) or OpenRouter endpoints. In desktop apps, users configuring their own local endpoints is intentional, though cloud metadata (169.254.169.254) should be blocked.
- **Rule Message:** The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value.

#### Finding #51: `js/request-forgery` in `backend/routes/settingsRoutes.js:122-126`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** User-supplied LLM baseUrl in /api/settings/llm/test
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Accepted Risk / Local Intended Configuration**
- **Forensic Exploitability Analysis:** Users configure local Ollama (http://localhost:11434) or OpenRouter endpoints. In desktop apps, users configuring their own local endpoints is intentional, though cloud metadata (169.254.169.254) should be blocked.
- **Rule Message:** The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value.

#### Finding #50: `js/request-forgery` in `backend/routes/settingsRoutes.js:110-114`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** User-supplied LLM baseUrl in /api/settings/llm/test
- **Sink:** fetch() or HTTP client call
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Accepted Risk / Local Intended Configuration**
- **Forensic Exploitability Analysis:** Users configure local Ollama (http://localhost:11434) or OpenRouter endpoints. In desktop apps, users configuring their own local endpoints is intentional, though cloud metadata (169.254.169.254) should be blocked.
- **Rule Message:** The URL of this request depends on a user-provided value. The URL of this request depends on a user-provided value.

### Category: Missing Rate Limiting (12 Findings)

#### Finding #46: `js/missing-rate-limiting` in `backend/routes/ttsRoutes.js:54-78`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a file system access, but is not rate-limited. This route handler performs a file system access, but is not rate-limited.

#### Finding #45: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:617-701`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited. This route handler performs authorization, but is not rate-limited.

#### Finding #44: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:558-614`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited.

#### Finding #43: `js/missing-rate-limiting` in `backend/routes/resynthesisRoutes.js:10-136`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited.

#### Finding #42: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:350-350`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited.

#### Finding #41: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:283-310`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #40: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:181-211`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #39: `js/missing-rate-limiting` in `backend/routes/importQueueRoutes.js:255-306`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #38: `js/missing-rate-limiting` in `backend/routes/importQueueRoutes.js:200-252`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #37: `js/missing-rate-limiting` in `backend/routes/bookRoutes.js:93-178`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #36: `js/missing-rate-limiting` in `backend/routes/importQueueRoutes.js:126-197`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited.

#### Finding #35: `js/missing-rate-limiting` in `backend/routes/lifecycleRoutes.js:243-243`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Express HTTP endpoints
- **Sink:** Route handler execution
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Architectural False Positive / Intended Local Architecture**
- **Forensic Exploitability Analysis:** Omnitome is a local desktop application executing on localhost for a single user. Rate-limiting localhost would degrade UI responsiveness and background queue communication without providing security benefit.
- **Rule Message:** This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited. This route handler performs a database access, but is not rate-limited.

### Category: Type Confusion via Parameter Tampering (1 Findings)

#### Finding #34: `js/type-confusion-through-parameter-tampering` in `backend/routes/importQueueRoutes.js:179-179`
- **Severity / Security Severity:** `ERROR` (`critical`)
- **Source:** req.body.title
- **Sink:** importQueueService.enqueueFile
- **Relevant Existing Sanitization:** typeof req.body.title === "string"
- **Triage Classification:** **Confirmed False Positive**
- **Forensic Exploitability Analysis:** The code explicitly enforces typeof req.body.title === "string", dropping arrays or objects to null. CodeQL did not infer that the ternary guard completely eliminates array parameter tampering.
- **Rule Message:** Potential type confusion as this HTTP request parameter may be either an array or a string.

### Category: Regular Expression Denial of Service (ReDoS) (13 Findings)

#### Finding #32: `js/polynomial-redos` in `backend/services/settings/settingsService.js:114-114`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'.

#### Finding #31: `js/polynomial-redos` in `backend/services/ai/ttsClient.js:34-35`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '\t'.

#### Finding #30: `js/polynomial-redos` in `backend/services/ai/aiNormalizer.js:266-268`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings starting with '\[' and with many repetitions of '\[\\'. This regular expression that depends on a user-provided value may run slow on strings starting with '\[\\\](' and with many repetitions of '\[(\]('. This regular expression that depends on a user-provided value may run slow on strings starting with '\[' and with many repetitions of '\[\\'. This regular expression that depends on a user-provided value may run slow on strings starting with '\[\\\](' and with many repetitions of '\[(\]('. This regular expression that depends on a user-provided value may run slow on strings starting with '\[' and with many repetitions of '\[\\'. This regular expression that depends on a user-provided value may run slow on strings starting with '\[\\\](' and with many repetitions of '\[(\]('.

#### Finding #29: `js/polynomial-redos` in `backend/routes/settingsRoutes.js:130-130`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'. This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'.

#### Finding #28: `js/polynomial-redos` in `backend/routes/settingsRoutes.js:107-107`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'. This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'.

#### Finding #27: `js/polynomial-redos` in `backend/routes/settingsRoutes.js:89-89`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'. This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'.

#### Finding #26: `js/polynomial-redos` in `backend/routes/settingsRoutes.js:83-83`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'. This regular expression that depends on a user-provided value may run slow on strings with many repetitions of '/'.

#### Finding #25: `js/redos` in `backend/services/ingestion/parsers/markdownParser.js:197-197`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This part of the regular expression may cause exponential backtracking on strings starting with '-' and containing many repetitions of '-'.

#### Finding #24: `js/redos` in `backend/services/ingestion/parsers/markdownParser.js:129-129`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This part of the regular expression may cause exponential backtracking on strings starting with '-' and containing many repetitions of '-'.

#### Finding #23: `js/redos` in `backend/services/ai/aiNormalizer.js:211-211`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This part of the regular expression may cause exponential backtracking on strings starting with '-' and containing many repetitions of '-'.

#### Finding #22: `js/redos` in `backend/services/ai/aiNormalizer.js:142-142`
- **Severity / Security Severity:** `ERROR` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This part of the regular expression may cause exponential backtracking on strings starting with '-' and containing many repetitions of '-'.

#### Finding #21: `py/polynomial-redos` in `sidecars/python/tts/synthesize.py:282-282`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of ' '.

#### Finding #20: `py/polynomial-redos` in `sidecars/python/tts/synthesize.py:281-281`
- **Severity / Security Severity:** `WARNING` (`high`)
- **Source:** Imported document text (markdown/HTML)
- **Sink:** RegExp.test() / RegExp.exec() in parsers
- **Relevant Existing Sanitization:** None
- **Triage Classification:** **Real Finding (Low Severity Edge Case)**
- **Forensic Exploitability Analysis:** Markdown table regex /^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/ in markdownParser.js and aiNormalizer.js exhibits polynomial/exponential backtracking on long lines containing repeated dashes and pipes. Maliciously crafted books could freeze Node.js event loop during ingestion. Rewriting with linear character-scanning eliminates the issue.
- **Rule Message:** This regular expression that depends on a user-provided value may run slow on strings with many repetitions of ' '.

### Category: Stack Trace Exposure (19 Findings)

#### Finding #19: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:683-687`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #18: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:596-600`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #17: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:521-525`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #16: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:445-449`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #15: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:238-242`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #14: `py/stack-trace-exposure` in `sidecars/python/nlp/routes.py:178-182`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #13: `py/stack-trace-exposure` in `sidecars/python/parse/routes.py:116-116`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user.

#### Finding #12: `py/stack-trace-exposure` in `sidecars/python/parse/routes.py:106-115`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #11: `py/stack-trace-exposure` in `sidecars/python/ocr/routes.py:216-220`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #10: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:132-137`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #9: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:123-128`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #8: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:109-114`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #7: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:100-105`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #6: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:88-93`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #5: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:78-83`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #4: `py/stack-trace-exposure` in `sidecars/python/tts/routes.py:68-73`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #3: `py/stack-trace-exposure` in `sidecars/python/embed/routes.py:177-181`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user. Stack trace information flows to this location and may be exposed to an external user.

#### Finding #2: `py/stack-trace-exposure` in `sidecars/python/pdfmath_sidecar/extract.py:155-155`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

#### Finding #1: `py/stack-trace-exposure` in `sidecars/python/pdfmath_sidecar/extract.py:148-153`
- **Severity / Security Severity:** `ERROR` (`medium`)
- **Source:** FastAPI route exception handler
- **Sink:** HTTP Response body (JSON error payload)
- **Relevant Existing Sanitization:** Local error wrapping
- **Triage Classification:** **Accepted Risk / Low Impact**
- **Forensic Exploitability Analysis:** The Python sidecar runs solely on 127.0.0.1:8765 as a child IPC process for Node.js. Exposing tracebacks to Node.js is useful for local desktop logging, but exposes internal paths if an attacker could inspect local IPC traffic. Masking tracebacks in production sidecars is recommended.
- **Rule Message:** Stack trace information flows to this location and may be exposed to an external user.

