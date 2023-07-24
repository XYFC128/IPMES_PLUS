package org.ipmes;

import org.json.*;

public class Preprocess {
    public static String extractNodeSignature(JSONObject nodeObj) {
        JSONObject properties = nodeObj.getJSONObject("properties");
        String type = properties.getString("type");
        String signatre = String.format("%s::", type);
        if (type.equals("Process")) {
            signatre += properties.getString("name");
        } else if (type.equals("Artifact")) {
            String subtype = properties.getString("subtype");
            signatre += String.format("%s::", subtype);
            if (subtype.equals("file") || subtype.equals("diectory")) {
                signatre += properties.getString("path");
            } else if (subtype.equals("network socket")) {
                signatre += String.format(
                        "%s:%s",
                        properties.getString("remote address"),
                        properties.getString("remote port")
                );
            }
        }
        return signatre;
    }

    public static String extractEdgeSignature(JSONObject edgeObj) {
        return edgeObj.getJSONObject("properties").getString("operation");
    }
}
