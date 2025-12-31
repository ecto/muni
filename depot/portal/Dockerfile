FROM nginx:alpine

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy static files
COPY index.html /usr/share/nginx/html/
COPY style.css /usr/share/nginx/html/
COPY depot-nav.js /usr/share/nginx/html/
COPY *.woff2 /usr/share/nginx/html/

EXPOSE 80
