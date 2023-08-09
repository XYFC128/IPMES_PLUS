package org.ipmes.pattern;

import org.json.JSONObject;

public interface SigExtractor {
    String extractNodeSignature(JSONObject nodeObj);

    String extractEdgeSignature(JSONObject edgeObj);
}
