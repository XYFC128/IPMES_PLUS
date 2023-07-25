package org.ipmes;

import org.json.*;

public class Preprocess {
    public static String extractNodeSignature(JSONObject nodeObj) {
        JSONObject properties = nodeObj.getJSONObject("properties");
        String type = properties.getString("type");
        String signature = String.format("%s::", type);
        if (type.equals("Process")) {
            signature += properties.getString("name");
        } else if (type.equals("Artifact")) {
            String subtype = properties.getString("subtype");
            signature += String.format("%s::", subtype);
            if (subtype.equals("file") || subtype.equals("directory")) {
                signature += properties.getString("path");
            } else if (subtype.equals("network socket")) {
                signature += String.format(
                        "%s:%s",
                        properties.getString("remote address"),
                        properties.getString("remote port"));
            }
        }
        return signature;
    }

    public static String extractEdgeSignature(JSONObject edgeObj) {
        return edgeObj.getJSONObject("properties").getString("operation");
    }
}
