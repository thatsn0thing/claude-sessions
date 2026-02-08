#!/usr/bin/env node

/**
 * Web Bridge for Claude Sessions
 * Translates HTTP requests to Unix socket daemon calls
 */

const http = require('http');
const net = require('net');
const fs = require('fs');
const path = require('path');
const os = require('os');

const PORT = process.env.PORT || 3030;
const SOCKET_PATH = path.join(os.homedir(), '.claude-sessions', 'daemon.sock');

// CORS headers for all responses
const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
  'Content-Type': 'application/json'
};

// Helper to call daemon via Unix socket
async function callDaemon(request) {
  return new Promise((resolve, reject) => {
    const client = net.createConnection(SOCKET_PATH);
    let responseData = '';

    const requestJson = JSON.stringify(request);
    
    client.on('connect', () => {
      client.write(requestJson + '\n');
    });

    client.on('data', (data) => {
      responseData += data.toString();
    });

    client.on('end', () => {
      try {
        if (!responseData.trim()) {
          reject(new Error('Empty response from daemon'));
          return;
        }
        const response = JSON.parse(responseData.trim());
        resolve(response);
      } catch (err) {
        console.error('Failed to parse daemon response:', err.message);
        console.error('Raw response data:', responseData);
        reject(new Error(`Invalid JSON response from daemon: ${err.message}`));
      }
    });

    client.on('error', (err) => {
      reject(err);
    });
  });
}

// Route handlers
const routes = {
  'GET /api/sessions': async () => {
    const response = await callDaemon({ type: 'list_sessions' });
    if (response.type === 'error') {
      throw new Error(response.message);
    }
    return response.sessions || [];
  },

  'POST /api/sessions': async (params, body) => {
    const { working_dir } = body;
    if (!working_dir) {
      throw new Error('working_dir is required');
    }
    const response = await callDaemon({ 
      type: 'start_session',
      working_dir 
    });
    if (response.type === 'error') {
      throw new Error(response.message);
    }
    return { id: response.session_id, log_path: response.log_path };
  },

  'DELETE /api/sessions/:id': async (params) => {
    const response = await callDaemon({ 
      type: 'stop_session',
      session_id: params.id 
    });
    if (response.type === 'error') {
      throw new Error(response.message);
    }
    return { success: true };
  },

  'GET /api/sessions/:id/logs': async (params) => {
    // For now, this isn't directly supported by the daemon
    // Clients should read the log file directly or we need to add support
    throw new Error('Log reading not yet implemented in web bridge');
  },

  'POST /api/sessions/:id/input': async (params, body) => {
    const { input } = body;
    if (!input) {
      throw new Error('input is required');
    }
    const response = await callDaemon({ 
      type: 'send_input',
      session_id: params.id,
      text: input
    });
    if (response.type === 'error') {
      throw new Error(response.message);
    }
    return { success: true };
  }
};

// Simple router
function matchRoute(method, url) {
  const urlPath = url.split('?')[0];
  
  for (const [routePattern, handler] of Object.entries(routes)) {
    const [routeMethod, routePath] = routePattern.split(' ');
    
    if (method !== routeMethod) continue;
    
    const paramNames = [];
    const regexPattern = routePath.replace(/:([^/]+)/g, (_, name) => {
      paramNames.push(name);
      return '([^/]+)';
    });
    
    const regex = new RegExp('^' + regexPattern + '$');
    const match = urlPath.match(regex);
    
    if (match) {
      const params = {};
      paramNames.forEach((name, i) => {
        params[name] = match[i + 1];
      });
      return { handler, params };
    }
  }
  
  return null;
}

// HTTP server
const server = http.createServer(async (req, res) => {
  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(200, CORS_HEADERS);
    res.end();
    return;
  }

  try {
    const route = matchRoute(req.method, req.url);
    
    if (!route) {
      res.writeHead(404, CORS_HEADERS);
      res.end(JSON.stringify({ error: 'Not found' }));
      return;
    }

    // Parse body for POST/DELETE
    let body = {};
    if (req.method === 'POST' || req.method === 'DELETE') {
      const chunks = [];
      for await (const chunk of req) {
        chunks.push(chunk);
      }
      const bodyStr = Buffer.concat(chunks).toString();
      if (bodyStr) {
        body = JSON.parse(bodyStr);
      }
    }

    const result = await route.handler(route.params, body);
    
    res.writeHead(200, CORS_HEADERS);
    res.end(JSON.stringify(result));
    
  } catch (err) {
    console.error('Error:', err);
    res.writeHead(500, CORS_HEADERS);
    res.end(JSON.stringify({ 
      error: err.message || 'Internal server error'
    }));
  }
});

server.listen(PORT, () => {
  console.log(`🌉 Claude Sessions Web Bridge running on http://localhost:${PORT}`);
  console.log(`📁 Socket: ${SOCKET_PATH}`);
  console.log(`\nAPI Endpoints:`);
  console.log(`  GET    /api/sessions          - List sessions`);
  console.log(`  POST   /api/sessions          - Create session`);
  console.log(`  DELETE /api/sessions/:id      - Delete session`);
  console.log(`  GET    /api/sessions/:id/logs - Get logs`);
  console.log(`  POST   /api/sessions/:id/input - Send input`);
});
