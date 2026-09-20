use super::App;
use crate::editor::EditorEntry;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PathKind {
    Relative,
    Absolute,
}

pub(crate) struct EditorPickerState {
    pub(super) open: bool,
    pub(super) editors: Vec<EditorEntry>,
    pub(super) index: usize,
}

impl App {
    pub(crate) fn open_help(&mut self) {
        self.help_open = true;
    }

    pub(crate) fn close_help(&mut self) {
        self.help_open = false;
    }

    pub(crate) fn is_help_open(&self) -> bool {
        self.help_open
    }

    pub(crate) fn open_path_popup(&mut self) {
        self.path_popup_open = true;
    }

    pub(crate) fn close_path_popup(&mut self) {
        self.path_popup_open = false;
        self.path_popup_hover = None;
        self.path_popup_rel_area = None;
        self.path_popup_abs_area = None;
        self.path_copy_flash = None;
    }

    pub(crate) fn is_path_popup_open(&self) -> bool {
        self.path_popup_open
    }

    pub(crate) fn relative_path_string(&self) -> String {
        if let Some(path) = self.filepath() {
            let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            std::env::current_dir()
                .ok()
                .and_then(|cwd| abs.strip_prefix(&cwd).ok().map(|p| p.display().to_string()))
                .unwrap_or_else(|| path.display().to_string())
        } else {
            "(stdin)".to_string()
        }
    }

    pub(crate) fn absolute_path_string(&self) -> String {
        if let Some(path) = self.filepath() {
            let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let abs_str = abs.display().to_string();
            abs_str
                .strip_prefix(r"\\?\")
                .unwrap_or(&abs_str)
                .to_string()
        } else {
            "(stdin)".to_string()
        }
    }

    pub(crate) fn copy_path_relative(&mut self) {
        let success = crate::clipboard::copy_to_clipboard(&self.relative_path_string());
        self.path_copy_flash = Some((PathKind::Relative, success, Instant::now()));
    }

    pub(crate) fn copy_path_absolute(&mut self) {
        let success = crate::clipboard::copy_to_clipboard(&self.absolute_path_string());
        self.path_copy_flash = Some((PathKind::Absolute, success, Instant::now()));
    }

    pub(crate) fn path_copy_flash(&self) -> Option<&(PathKind, bool, Instant)> {
        self.path_copy_flash.as_ref()
    }

    pub(crate) fn copy_path_to_clipboard_relative(&mut self) {
        let success = crate::clipboard::copy_to_clipboard(&self.relative_path_string());
        if success {
            self.set_path_flash(super::PathFlash::RelativeCopied);
        } else {
            self.set_path_flash(super::PathFlash::CopyFailed);
        }
    }

    pub(crate) fn copy_path_to_clipboard_absolute(&mut self) {
        let success = crate::clipboard::copy_to_clipboard(&self.absolute_path_string());
        if success {
            self.set_path_flash(super::PathFlash::AbsoluteCopied);
        } else {
            self.set_path_flash(super::PathFlash::CopyFailed);
        }
    }

    pub(crate) fn is_popup_open(&self) -> bool {
        self.help_open
            || self.path_popup_open
            || self.file_picker.open
            || self.theme_picker.open
            || self.editor_picker.open
            || self.is_picker_loading()
            || self.is_picker_load_failed()
    }

    pub(crate) fn open_editor_picker(&mut self) {
        let editors = crate::editor::scan_available_editors();
        let current = self
            .editor_config
            .as_deref()
            .map(crate::editor::binary_name);
        let index = current
            .and_then(|bin| {
                editors
                    .iter()
                    .position(|e| crate::editor::binary_name(&e.command) == bin)
            })
            .unwrap_or(0);
        self.editor_picker.editors = editors;
        self.editor_picker.index = index;
        self.editor_picker.open = true;
    }

    pub(crate) fn close_editor_picker(&mut self) {
        if let Some(entry) = self.editor_picker.editors.get(self.editor_picker.index) {
            self.editor_config = Some(entry.command.clone());
        }
        self.editor_picker.open = false;
    }

    pub(crate) fn cancel_editor_picker(&mut self) {
        self.editor_picker.open = false;
    }

    pub(crate) fn is_editor_picker_open(&self) -> bool {
        self.editor_picker.open
    }

    pub(crate) fn move_editor_picker_up(&mut self) {
        let len = self.editor_picker.editors.len();
        if len > 0 {
            self.editor_picker.index = (self.editor_picker.index + len - 1) % len;
        }
    }

    pub(crate) fn move_editor_picker_down(&mut self) {
        let len = self.editor_picker.editors.len();
        if len > 0 {
            self.editor_picker.index = (self.editor_picker.index + 1) % len;
        }
    }

    pub(crate) fn editor_picker_index(&self) -> usize {
        self.editor_picker.index
    }

    pub(crate) fn editor_picker_entries(&self) -> &[EditorEntry] {
        &self.editor_picker.editors
    }
}
