/**
 * ForgeFabrik DevStudio — AWS ECS Fargate Infrastructure
 *
 * Ressourcen:
 *   - ECR Repository        (Docker-Image-Registry)
 *   - VPC + Subnetze        (via awsx.ec2.Vpc — public + private)
 *   - Security Groups       (ALB + ECS-Task)
 *   - ECS Cluster           (Fargate)
 *   - IAM Roles             (Execution + Task)
 *   - ECS Task Definition   (Fargate, configurable CPU/Memory)
 *   - Application Load Balancer + Target Group + Listener
 *   - ECS Service           (Fargate, auto-scaling-fähig)
 *
 * Konfiguration (pulumi config set <key> <value>):
 *   containerPort   Interner Container-Port      (default: 8080)
 *   cpu             Fargate Task CPU units       (default: 256)
 *   memory          Fargate Task Memory (MB)     (default: 512)
 *   desiredCount    Gewünschte Task-Anzahl        (default: 1)
 *   appImage        Vollständiges Image-Tag       (default: ECR-Repo:latest)
 */

import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";

// ── Konfiguration ──────────────────────────────────────────────────────────

const config        = new pulumi.Config();
const stack         = pulumi.getStack();
const project       = pulumi.getProject();

const containerPort  = config.getNumber("containerPort") ?? 8080;
const cpu            = config.getNumber("cpu")           ?? 256;
const memory         = config.getNumber("memory")        ?? 512;
const desiredCount   = config.getNumber("desiredCount")  ?? 1;
const appImageTag    = config.get("appImage");           // optional override

// Gemeinsame Ressourcen-Tags
const tags: Record<string, string> = {
    Project:     project,
    Stack:       stack,
    ManagedBy:   "pulumi",
    Application: "devstudio",
};

// ── ECR Repository ─────────────────────────────────────────────────────────

const repo = new aws.ecr.Repository("devstudio-repo", {
    imageTagMutability: "MUTABLE",
    imageScanningConfiguration: { scanOnPush: true },
    forceDelete: true,
    tags,
});

// Lifecycle-Policy: nur die letzten 10 Images behalten
new aws.ecr.LifecyclePolicy("devstudio-repo-lifecycle", {
    repository: repo.name,
    policy: JSON.stringify({
        rules: [{
            rulePriority: 1,
            description:  "Halte die letzten 10 Images",
            selection: {
                tagStatus:   "any",
                countType:   "imageCountMoreThan",
                countNumber: 10,
            },
            action: { type: "expire" },
        }],
    }),
});

// Image-Referenz: entweder aus Config oder ECR-Repo mit :latest
const imageUri = appImageTag
    ? pulumi.output(appImageTag)
    : pulumi.interpolate`${repo.repositoryUrl}:latest`;

// ── VPC ────────────────────────────────────────────────────────────────────

const vpc = new awsx.ec2.Vpc("devstudio-vpc", {
    numberOfAvailabilityZones: 2,
    subnetSpecs: [
        { type: awsx.ec2.SubnetType.Public,  cidrMask: 24 },
        { type: awsx.ec2.SubnetType.Private, cidrMask: 24 },
    ],
    enableDnsHostnames: true,
    enableDnsSupport:   true,
    tags,
});

// ── Security Groups ────────────────────────────────────────────────────────

// ALB: nimmt HTTP (80) aus dem Internet an
const albSg = new aws.ec2.SecurityGroup("devstudio-alb-sg", {
    vpcId:       vpc.vpcId,
    description: "DevStudio ALB — erlaubt HTTP-Eingehend aus dem Internet",
    ingress: [{
        protocol:   "tcp",
        fromPort:   80,
        toPort:     80,
        cidrBlocks: ["0.0.0.0/0"],
        description: "HTTP von überall",
    }],
    egress: [{
        protocol:   "-1",
        fromPort:   0,
        toPort:     0,
        cidrBlocks: ["0.0.0.0/0"],
        description: "Alle ausgehenden Verbindungen",
    }],
    tags: { ...tags, Name: "devstudio-alb-sg" },
});

// ECS-Task: nimmt Anfragen nur vom ALB an
const taskSg = new aws.ec2.SecurityGroup("devstudio-task-sg", {
    vpcId:       vpc.vpcId,
    description: "DevStudio ECS-Task — erlaubt Eingehend nur vom ALB",
    ingress: [{
        protocol:       "tcp",
        fromPort:       containerPort,
        toPort:         containerPort,
        securityGroups: [albSg.id],
        description:    "Vom ALB weitergeleiteter Traffic",
    }],
    egress: [{
        protocol:   "-1",
        fromPort:   0,
        toPort:     0,
        cidrBlocks: ["0.0.0.0/0"],
        description: "Alle ausgehenden (z. B. Anthropic API, ECR Pull)",
    }],
    tags: { ...tags, Name: "devstudio-task-sg" },
});

// ── ECS Cluster ────────────────────────────────────────────────────────────

const cluster = new aws.ecs.Cluster("devstudio-cluster", {
    settings: [{
        name:  "containerInsights",
        value: "enabled",
    }],
    tags,
});

// ── IAM Rollen ─────────────────────────────────────────────────────────────

// Execution Role: ECS-Agent zieht Images aus ECR und schreibt in CloudWatch
const executionRole = new aws.iam.Role("devstudio-exec-role", {
    assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({
        Service: "ecs-tasks.amazonaws.com",
    }),
    managedPolicyArns: [
        "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy",
    ],
    tags,
});

// Task Role: Berechtigungen des laufenden Containers
const taskRole = new aws.iam.Role("devstudio-task-role", {
    assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({
        Service: "ecs-tasks.amazonaws.com",
    }),
    tags,
});

// CloudWatch Log Group für Container-Logs
const logGroup = new aws.cloudwatch.LogGroup("devstudio-logs", {
    retentionInDays: 7,
    tags,
});

// ── ECS Task Definition ────────────────────────────────────────────────────

const taskDef = new aws.ecs.TaskDefinition("devstudio-task", {
    family:                  "devstudio",
    cpu:                     cpu.toString(),
    memory:                  memory.toString(),
    networkMode:             "awsvpc",
    requiresCompatibilities: ["FARGATE"],
    executionRoleArn:        executionRole.arn,
    taskRoleArn:             taskRole.arn,
    containerDefinitions: pulumi.all([imageUri, logGroup.name]).apply(
        ([image, logGroupName]) => JSON.stringify([{
            name:      "devstudio",
            image,
            essential: true,
            portMappings: [{
                containerPort,
                protocol: "tcp",
            }],
            environment: [
                { name: "DEVSTUDIO_SERVER_HOST", value: "0.0.0.0" },
                { name: "DEVSTUDIO_SERVER_PORT", value: containerPort.toString() },
                { name: "DEVSTUDIO_LOG_JSON",    value: "true" },
                { name: "DEVSTUDIO_LOG_LEVEL",   value: "info" },
            ],
            logConfiguration: {
                logDriver: "awslogs",
                options: {
                    "awslogs-group":         logGroupName,
                    "awslogs-region":        aws.config.region ?? "eu-central-1",
                    "awslogs-stream-prefix": "devstudio",
                },
            },
            healthCheck: {
                command:     ["CMD-SHELL", `curl -sf http://localhost:${containerPort}/health || exit 1`],
                interval:    30,
                timeout:     5,
                retries:     3,
                startPeriod: 10,
            },
        }]),
    ),
    tags,
});

// ── Application Load Balancer ──────────────────────────────────────────────

const alb = new aws.lb.LoadBalancer("devstudio-alb", {
    internal:                    false,
    loadBalancerType:            "application",
    securityGroups:              [albSg.id],
    subnets:                     vpc.publicSubnetIds,
    enableDeletionProtection:    false,
    enableHttp2:                 true,
    tags,
});

const targetGroup = new aws.lb.TargetGroup("devstudio-tg", {
    port:              containerPort,
    protocol:          "HTTP",
    targetType:        "ip",
    vpcId:             vpc.vpcId,
    deregistrationDelay: 30,
    healthCheck: {
        enabled:            true,
        path:               "/health",
        protocol:           "HTTP",
        matcher:            "200",
        interval:           30,
        timeout:            5,
        healthyThreshold:   2,
        unhealthyThreshold: 3,
    },
    tags,
});

const listener = new aws.lb.Listener("devstudio-listener", {
    loadBalancerArn: alb.arn,
    port:            80,
    protocol:        "HTTP",
    defaultActions: [{
        type:           "forward",
        targetGroupArn: targetGroup.arn,
    }],
    tags,
});

// ── ECS Service ────────────────────────────────────────────────────────────

const service = new aws.ecs.Service("devstudio-svc", {
    cluster:        cluster.arn,
    taskDefinition: taskDef.arn,
    desiredCount,
    launchType:     "FARGATE",

    networkConfiguration: {
        subnets:        vpc.privateSubnetIds,
        securityGroups: [taskSg.id],
        assignPublicIp: false,
    },

    loadBalancers: [{
        targetGroupArn: targetGroup.arn,
        containerName:  "devstudio",
        containerPort,
    }],

    // Reibungsloses Deployment: erst neue Tasks, dann alte
    deploymentMinimumHealthyPercent: 100,
    deploymentMaximumPercent:        200,

    deploymentCircuitBreaker: {
        enable:   true,
        rollback: true,
    },

    tags,
}, {
    // Nur updaten, wenn Listener bereit ist
    dependsOn: [listener],
});

// ── Outputs ────────────────────────────────────────────────────────────────

export const ecrRepositoryUrl    = repo.repositoryUrl;
export const albDnsName          = alb.dnsName;
export const serviceUrl          = pulumi.interpolate`http://${alb.dnsName}`;
export const healthEndpoint      = pulumi.interpolate`http://${alb.dnsName}/health`;
export const ecsClusterName      = cluster.name;
export const ecsServiceName      = service.name;
export const vpcId               = vpc.vpcId;
export const logGroupName        = logGroup.name;
