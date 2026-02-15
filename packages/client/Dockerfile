# Build stage
FROM node:20-slim AS builder
WORKDIR /app
COPY package.json package-lock.json* ./
COPY packages/shared ./packages/shared
COPY packages/client ./packages/client
RUN npm install --workspaces
RUN npm run build -w @open-haxball/client

# Runtime stage
FROM nginx:alpine
COPY --from=builder /app/packages/client/index.html /usr/share/nginx/html/
COPY --from=builder /app/packages/client/dist /usr/share/nginx/html/dist
EXPOSE 80
