# Implementation Plan

- [x] 1. Create domain detection module
  - Create `src/crawler/domain_detector.rs` with domain-to-mode mapping functionality
  - Implement `DomainDetector` struct with SPA/SSR domain classification methods
  - Add configuration loading from YAML file for domain lists
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 2. Implement HTML to Markdown converter
  - Create `src/crawler/html_converter.rs` with HTML parsing and conversion logic
  - Implement tag conversion methods for h1, h2, p, ul, li, ol, a, img, strong, em, br, blockquote
  - Add text processing functions to generate readable Markdown output
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 3. Create file management system
  - Create `src/crawler/file_manager.rs` with filename generation and file saving logic
  - Implement URL-to-filename conversion with sanitization
  - Add output directory management and duplicate filename handling
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 4. Enhance existing crawler with Chrome mode support
  - Modify `src/crawler/robots.rs` to support Chrome browser mode for SPA websites
  - Update `fetch_html_from_urls` function to accept and use fetch mode parameter
  - Add Chrome browser configuration and initialization for JavaScript rendering
  - _Requirements: 1.2, 1.3, 4.4_

- [x] 5. Integrate new components into main pipeline
  - Update `src/crawler/crawler.rs` to use domain detection for mode selection
  - Integrate HTML converter and file manager into the crawling workflow
  - Add proper error handling and logging throughout the pipeline
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [x] 6. Create configuration system
  - Create `config/crawler.yaml` with SPA/SSR domain lists and crawler settings
  - Update module declarations in `src/crawler/mod.rs` for new components
  - Add configuration loading logic to main crawler orchestration
  - _Requirements: 1.4, 5.3, 5.4_

- [x] 7. Add comprehensive error handling and logging
  - Implement custom error types for different failure scenarios
  - Add structured logging throughout all modules with appropriate log levels
  - Implement graceful error recovery and fallback mechanisms
  - _Requirements: 4.3, 4.5, 5.4_

- [x] 8. Create comprehensive configuration and error management
  - Create `src/crawler/config.rs` with complete configuration structure
  - Create `src/crawler/errors.rs` with comprehensive error types and recovery strategies
  - Create `src/crawler/logging.rs` with structured logging and performance metrics
  - _Requirements: 5.3, 5.4, 4.3, 4.5_

- [ ]* 9. Create unit tests for core functionality
  - Write unit tests for domain detector with various domain patterns
  - Create tests for HTML converter with sample HTML inputs and expected Markdown outputs
  - Add tests for file manager filename generation and sanitization
  - _Requirements: 2.3, 3.3, 5.1_

- [ ] 10. Test and validate complete pipeline


  - Test the integrated pipeline with sample websites (both SPA and SSR)
  - Verify Markdown file generation and content quality
  - Validate error handling with edge cases and malformed inputs
  - _Requirements: 4.1, 4.2, 4.5, 5.1, 5.2_
