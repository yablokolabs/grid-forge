# Kubernetes deployment notes

This folder is a placeholder for Helm/Kustomize manifests. A production deployment should include:

- Separate API and worker deployments
- External Postgres with pgvector
- Managed object storage bucket
- Secret manager integration
- Network policies
- Pod security contexts
- Horizontal pod autoscaling for API
- Worker queue scaling based on ingestion backlog
