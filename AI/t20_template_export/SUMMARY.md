# T20 — Integrate template export UI flow

## Task goal
Extend the template export flow so users can select a custom template XML file
(e.g. a Sunflow or POV-Ray renderer template) instead of always using the
built-in text-dump template.

## Approach

### Model change (`app.rs`)
New field: `template_path: Option<PathBuf>` — `None` means "use built-in".

### New `AppMsg` variants
| Variant | Action |
|---|---|
| `SetTemplateFile` | Opens a file-open dialog to pick a `.xml` template |
| `TemplateFileSet(PathBuf)` | Stores the chosen path in model, updates status |
| `ClearTemplateFile` | Resets `template_path` to `None` |

### File menu additions (under the export section)
- **Set Template File…** — opens file dialog filtered to `*.xml`
- **Clear Template File** — reverts to built-in

### `export_template` helper changes
The existing helper was upgraded from a single hardcoded template to:
1. Check `self.template_path`:
   - `Some(path)` → read XML from disk; error if file is missing.
   - `None` → use the embedded built-in plain-text template.
2. Call `Template::from_xml(&template_xml)` (passing the dynamic string
   rather than the compile-time constant).
3. Status bar on success shows both the output filename and the template used:
   `"Template exported: scene.txt (template: built-in)"` or
   `"Template exported: scene.sunflow (template: sunflow.xml)"`.

### Existing file-dialog flow preserved
The `ExportTemplate` → `ExportTemplatePicked` two-message chain is unchanged.
`ExportTemplate` triggers the GTK file-save dialog; when the user picks a
destination path `ExportTemplatePicked` calls `export_template(&path)` which
now uses the stored `template_path`.

## Result
Complete. Users can:
1. **Set Template File…** (File menu) to choose any template XML.
2. **Export Template…** to write the scene using the active template.
3. **Clear Template File** to revert to the built-in text dump.

The chosen template persists for the lifetime of the session (not saved across
app restarts — a future enhancement).

## Status
Complete. No new tests (template-selection is pure UI plumbing; existing
template exporter tests still cover the core export logic in T11).
