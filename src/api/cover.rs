// Cover functionality is now handled inline in main.rs Commands::Cover branch,
// which builds the GenerateRequest directly with full control over all fields
// (task, generation_type, is_remix, vocal_gender, control_sliders, hCaptcha).
// The old SunoClient::cover() wrapper is no longer used.
