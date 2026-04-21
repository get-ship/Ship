---
name: ship-artifacts
stable-id: ship-artifacts
description: Reference for Ship's artifact model — the files API that agents and users operate through. Early-stage; shape is designed, implementation is partial.
tags: [ship, artifacts, storage, files, extension]
authors: [ship]
---

# Ship Artifacts

An artifact in Ship is a file Ship manages. Artifacts are the storage primitive — agents produce them, users consume them, and Ship controls where they live, how long they last, and how they sync. Three scopes: **ephemeral**, **project**, **global**. In the cloud, project and global artifacts back to S3.

- **[Overview](/reference/artifacts/)** — the intended model, three-scope split, current state, and what's aspirational vs implemented.

This doc is internal-audience while the model is being hammered out. It'll move public once the API stabilizes.
