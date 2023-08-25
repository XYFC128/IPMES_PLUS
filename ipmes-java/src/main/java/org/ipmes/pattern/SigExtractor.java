package org.ipmes.pattern;

import org.json.JSONObject;

/**
 * Abstract interface to extract signature from pattern format.
 */
public interface SigExtractor {
    String extractNodeSignature(JSONObject nodeObj);

    String extractEdgeSignature(JSONObject edgeObj);
}
