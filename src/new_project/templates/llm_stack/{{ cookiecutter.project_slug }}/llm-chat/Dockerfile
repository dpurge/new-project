FROM node:25-alpine3.23 AS build

WORKDIR /app

COPY package.json ./
RUN npm install

COPY . .
RUN npm run build

FROM nginx:stable-alpine3.23

ENV LLM_API_ORIGIN=http://llm-api:{{ cookiecutter.api_port }} \
    DOLLAR=$

COPY nginx.conf.template /etc/nginx/templates/default.conf.template
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
