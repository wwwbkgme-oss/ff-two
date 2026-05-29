# ForgeFabrik DevStudio — AWS ECS Fargate Infrastructure

Pulumi TypeScript IaC für den DevStudio-Server auf AWS ECS Fargate.

## Ressourcen

| Ressource | Beschreibung |
|---|---|
| ECR Repository | Docker-Image-Registry mit Lifecycle-Policy (max. 10 Images) |
| VPC (awsx) | 2 AZs, public + private Subnetze |
| Security Groups | ALB-SG (Port 80 öffentlich) + Task-SG (nur vom ALB) |
| ECS Cluster | Fargate, Container Insights aktiviert |
| IAM Roles | Execution Role (ECR Pull, CloudWatch) + Task Role |
| CloudWatch Logs | Log Group mit 7 Tagen Retention |
| Task Definition | Fargate, konfigurierbare CPU/Memory, Health Check |
| Application Load Balancer | Internet-facing, HTTP |
| ECS Service | Fargate, private Subnetze, Circuit Breaker + Rollback |

## Voraussetzungen

- [Pulumi CLI](https://www.pulumi.com/docs/get-started/install/) ≥ v3
- Node.js ≥ 18
- AWS-Credentials (`aws configure` oder `AWS_*` ENV-Variablen)

## Deployment

```bash
# Abhängigkeiten installieren
npm install

# Stack initialisieren (einmalig)
pulumi stack init dev

# Konfiguration setzen (oder Pulumi.dev.yaml verwenden)
pulumi config set aws:region eu-central-1

# Vorschau
pulumi preview

# Deployen
pulumi up
```

## Konfiguration

| Key | Beschreibung | Default |
|---|---|---|
| `aws:region` | AWS-Region | `eu-central-1` |
| `containerPort` | Container-Port | `8080` |
| `cpu` | Fargate CPU Units | `256` |
| `memory` | Fargate Memory (MB) | `512` |
| `desiredCount` | Gewünschte Task-Anzahl | `1` |
| `appImage` | Vollständiges Image-Tag (optional) | ECR Repo:latest |

```bash
pulumi config set forgefabrik-devstudio:cpu 512
pulumi config set forgefabrik-devstudio:desiredCount 2
```

## Outputs

Nach `pulumi up` stehen folgende Outputs zur Verfügung:

```bash
pulumi stack output serviceUrl      # http://<alb-dns>
pulumi stack output healthEndpoint  # http://<alb-dns>/health
pulumi stack output ecrRepositoryUrl
pulumi stack output logGroupName
```

## Docker-Image pushen

```bash
# ECR-Repository-URL ermitteln
REPO=$(pulumi stack output ecrRepositoryUrl)

# AWS ECR Login
aws ecr get-login-password --region eu-central-1 \
  | docker login --username AWS --password-stdin "$REPO"

# Image bauen und pushen
docker build -t "$REPO:latest" ../
docker push "$REPO:latest"
```

## Teardown

```bash
pulumi destroy
pulumi stack rm dev
```
