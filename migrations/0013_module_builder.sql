-- Module builder: reference fields, computed fields, module grouping.
-- reference: _meta_field.ref_entity points at the target entity id.
-- computed: _meta_field.computed_expr holds a `{field}` template.
-- module: _meta_entity.module groups entities in the sidebar.
ALTER TABLE _meta_field ADD COLUMN ref_entity TEXT;
ALTER TABLE _meta_field ADD COLUMN computed_expr TEXT;
ALTER TABLE _meta_entity ADD COLUMN module TEXT;
