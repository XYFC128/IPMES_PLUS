package org.ipmes.pattern;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.Reader;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Optional;
import java.util.stream.Collectors;

/**
 * TemporalRelation is a DAG describing the temporal relation of pattern edges.
 * <p>
 * The meaning of the elements in the graph:
 * <ul>
 * <li>
 * Nodes: each node represents a pattern edge. The id of a node
 * corresponds to the pattern edge id it representing.
 * </li>
 * <li>
 * Arc: each arc in the graph represents a dependency relation. The
 * end node depends on the start node. Also, the start node is the
 * parent of the end node.
 * </li>
 * <li>
 * Root: a virtual node with id -1, not representing any edge. If an
 * pattern edge has no dependency, it depends on the root.
 * </li>
 * </ul>
 * </p>
 */
public class TemporalRelation {
    ArrayList<ArrayList<Integer>> parents;
    ArrayList<ArrayList<Integer>> child;

    public TemporalRelation(ArrayList<ArrayList<Integer>> parents, ArrayList<ArrayList<Integer>> child) {
        this.parents = parents;
        this.child = child;
    }

    static ArrayList<Integer> parseArrayOfEdgeId(JSONArray array) {
        ArrayList<Integer> res = new ArrayList<>();
        for (int i = 0; i < array.length(); ++i) {
            Object obj = array.get(i);
            Integer item = -1;
            if (obj instanceof Integer) {
                item = (Integer) obj;
            }
            res.add(item);
        }
        return res;
    }

    /**
     * Construct the DependencyGraph from an orels file or string.
     *
     * @param orelsReader reader of orels file
     * @return an Optional of DependencyGraph if the parsing succeed, empty
     *         otherwise
     */
    public static Optional<TemporalRelation> parse(Reader orelsReader) {
        HashMap<Integer, ArrayList<Integer>> parentsMap = new HashMap<>();
        HashMap<Integer, ArrayList<Integer>> childMap = new HashMap<>();
        try {
            BufferedReader orelsBuf = new BufferedReader(orelsReader);
            String content = orelsBuf.lines().collect(Collectors.joining());
            orelsBuf.close();

            JSONObject graphObj = new JSONObject(content);
            Iterator<String> keys = graphObj.keys();
            while (keys.hasNext()) {
                String key = (String) keys.next();
                JSONObject curObj = graphObj.getJSONObject(key);

                int id = -1;
                if (!key.equals("root"))
                    id = Integer.parseInt(key);
                id += 1;

                parentsMap.put(id, parseArrayOfEdgeId(curObj.getJSONArray("parents")));
                childMap.put(id, parseArrayOfEdgeId(curObj.getJSONArray("children")));
            }
        } catch (Exception e) {
            return Optional.empty();
        }

        int numEdges = parentsMap.size();
        ArrayList<ArrayList<Integer>> parentsList = new ArrayList<>(numEdges);
        ArrayList<ArrayList<Integer>> childsList = new ArrayList<>(numEdges);
        for (int i = 0; i < numEdges; ++i) {
            parentsList.add(i, parentsMap.get(i));
            childsList.add(i, childMap.get(i));
        }
        return Optional.of(new TemporalRelation(parentsList, childsList));
    }

    /**
     * Get the dependencies of a pattern edge.
     * 
     * @param eid the id of the pattern edge, or -1 for root
     * @return a list of pattern edge id
     */
    public ArrayList<Integer> getParents(Integer eid) {
        return this.parents.get(eid + 1);
    }

    /**
     * Get all the edges depending on the given pattern edge.
     * 
     * @param eid the id of the pattern edge, or -1 for root
     * @return a list of pattern edge id
     */
    public ArrayList<Integer> getChildren(Integer eid) {
        return this.child.get(eid + 1);
    }
}
