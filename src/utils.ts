import { GraphData } from './type';

export function genGraph(node_num: number, edge_num: number): GraphData {
  return {
    nodes: [...Array(node_num)].fill({}),
    links: [...Array(edge_num)].map(() => {
      const from = (Math.random() * node_num) | 0;
      let to = (Math.random() * node_num) | 0;
      if (to === from) {
        to = (from + 1) % node_num;
      }
      return {
        from,
        to,
      };
    }),
  };
}
