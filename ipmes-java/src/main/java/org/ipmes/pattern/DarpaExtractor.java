package org.ipmes.pattern;

import org.json.JSONObject;

/**
 * Extract signatures from the pattern in darpa dataset.
 */
public class DarpaExtractor implements SigExtractor {
    public String extractNodeSignature(JSONObject nodeObj) {
        JSONObject properties = nodeObj.getJSONObject("properties");
        String type;
        if (properties.has("f")) {
            type = "OBJECT_SOCKET";
        } else if (properties.has("UnnamedPipeObject_baseObject_epoch")) {
            type = "OBJECT_UNNAMEPIPE";
        } else if (properties.has("type")) {
            type = properties.getString("type");
        } else {
            return  "OTHER";
        }

        String signature = String.format("%s::", type);
        if (type.equals("SUBJECT_PROCESS") || type.equals("SUBJECT_UNIT")) {
            signature += properties.getString("Subject_properties_map_name");
        } else if (type.equals("FILE_OBJECT_CHAR") || type.equals("FILE_OBJECT_FILE") || type.equals("FILE_OBJECT_DIR")) {
            signature += properties.getString("path");
        } else if (type.equals("OBJECT_SOCKET")) {
            Object port = properties.get("NetFlowObject_remotePort");
            String portStr = "";
            if (port instanceof String)
                portStr = (String) port;
            else if (port instanceof Integer)
                portStr = Integer.toString((Integer) port);
            signature += String.format("%s:%s",
                    properties.getString("NetFlowObject_remoteAddress"),
                    portStr);
        } else if (type.equals("SRCSINK_UNKNOWN")) {
            signature += Long.toString(properties.getLong("SrcSinkObject_pid"));
        }
        return signature;
    }

    public String extractEdgeSignature(JSONObject edgeObj) {
        return edgeObj.getJSONObject("properties").getString("type");
    }
}
