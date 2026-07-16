# Definitive release procedure

1. Confirm a clean committed revision and run `make check` and `make evidence`.
2. Run the Qiskit differential adapter in its pinned isolated environment.
3. Recollect the ten-repetition benchmark protocol and regenerate plots.
4. Render and visually inspect the research paper.
5. Run `packaging/scripts/build-release.sh` from the repository root.
6. Verify the source archive, binary archives, SBOM, and SHA-256 manifest on macOS and Linux.
7. Create the single definitive Git tag only after the release audit passes.
8. Publish the GitHub release, archive the exact source and evidence on Zenodo, record the DOI in `CITATION.cff`, and publish the Archeum Studios product page.

GitHub, Zenodo, DOI registration, signing credentials, and public product-page publication require project-owner accounts and are intentionally not automated by local build scripts.
