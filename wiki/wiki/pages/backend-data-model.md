---
title: Backend Data Model
tags: [backend, django, database, api]
sources: [patchlang-design-guide/backend]
updated: 2026-04-16
---

# Backend Data Model

**Source:** `docs/patchlang-design-guide/backend.md`
**Type:** Reference — Django models and API endpoints

## Summary

The Django REST API stores projects as a tree of `ProjectPage` rows — one row per canvas level. No signal flow content lives in the `Project` model itself (metadata only). Version history uses a `ProjectSnapshot` + `PageVersion` table to avoid monolithic JSON blobs.

---

## Models

### `Project` — metadata container

```python
class Project(models.Model):
    facility = ForeignKey('organizations.Facility', on_delete=CASCADE)
    name = CharField(max_length=255)
    slug = SlugField(max_length=255)
    manifest = JSONField(default=dict, blank=True)         # project.json contents
    project_summary = JSONField(default=dict, blank=True)  # for list views / search
    thumbnail_url = URLField(blank=True, null=True)
    created_at = DateTimeField(auto_now_add=True)
    updated_at = DateTimeField(auto_now=True)
    deleted_at = DateTimeField(null=True, blank=True)
```

Holds only metadata. No signal flow content.

### `ProjectPage` — one row per canvas level

```python
class ProjectPage(models.Model):
    project = ForeignKey(Project, related_name='pages', on_delete=CASCADE)
    parent = ForeignKey('self', null=True, blank=True, related_name='children', on_delete=CASCADE)
    path = CharField(max_length=500, db_index=True)    # e.g., "buildings/foh"
    name = CharField(max_length=255)                    # human-readable: "Front of House"
    patch_content = TextField(default='', blank=True)
    layout_json = JSONField(default=dict, blank=True)
    parse_result = JSONField(default=dict, blank=True)
    sort_order = PositiveIntegerField(default=0)
    created_at = DateTimeField(auto_now_add=True)
    updated_at = DateTimeField(auto_now=True)
```

Root page has `parent=None`. `path` matches the file path on disk.

### `LibraryFile` — shared templates

```python
class LibraryFile(models.Model):
    facility = ForeignKey('organizations.Facility', on_delete=CASCADE)
    name = CharField(max_length=255)
    slug = SlugField(max_length=255)
    patch_content = TextField()
    created_at = DateTimeField(auto_now_add=True)
    updated_at = DateTimeField(auto_now=True)
```

Scoped to a facility. Shared across projects.

### `ProjectSnapshot` + `PageVersion` — version history

```python
class ProjectSnapshot(models.Model):
    project = ForeignKey(Project, related_name='snapshots', on_delete=CASCADE)
    version_number = PositiveIntegerField()
    message = CharField(max_length=500, blank=True)
    created_at = DateTimeField(auto_now_add=True)
    created_by = ForeignKey(User, null=True, on_delete=SET_NULL)

class PageVersion(models.Model):
    snapshot = ForeignKey(ProjectSnapshot, related_name='page_versions', on_delete=CASCADE)
    page = ForeignKey(ProjectPage, on_delete=CASCADE)
    patch_content = TextField()
    layout_json = JSONField(default=dict)
```

Per-page version rows — not monolithic JSON blobs. Restoring a snapshot updates each page individually. Diffing two snapshots compares page versions row by row.

---

## API Endpoints

```
GET  /projects/{id}/                    → Project metadata + root page content
GET  /projects/{id}/pages/              → Flat list of all pages (for sidebar tree)
GET  /projects/{id}/pages/{path}/       → Single page content (drill-down load)
PUT  /projects/{id}/pages/{path}/       → Save one level
POST /projects/{id}/snapshots/          → Create snapshot (captures all pages)
GET  /projects/{id}/snapshots/{ver}/    → Retrieve snapshot with page versions
GET  /facilities/{id}/libraries/        → List shared library files
```

---

## Migration Strategy

**Phase 1 (additive, no breaking changes):**
1. Create `ProjectPage`, `LibraryFile`, `PageVersion` tables
2. Data migration: for each existing `Project`, create a `ProjectPage` with `parent=None`, copying `patch_content` and `layout_json`
3. Keep deprecated `patch_content` and `layout_json` fields on `Project` during transition
4. New page-based API endpoints coexist alongside existing endpoints

**Phase 2 (after frontend migration):**
1. Frontend switches to page-based endpoints
2. Remove deprecated fields from `Project`
3. Remove old snapshot format

---

## Why One Row Per Canvas Level

Frontend loads one level at a time. Per-page rows match:
- Loading pattern — one query per level
- Save pattern — one row per save
- Concurrency — two users editing different rooms don't conflict (row-level locking)

## Relation to Other Wiki Pages

- [[project-structure]] — how multi-file projects map to this schema
- [[python-api]] — `patchlang_python` functions used by the backend for validation
- [[frontend-integration]] — API endpoints the frontend calls during loading
