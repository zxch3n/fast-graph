export interface Node {
  id?: string | number;
  size?: number;
}

export enum DefaultNodeValue {
  Size = 5,
}

export interface Link {
  from: string | number;
  to: string | number;
}

export interface GraphData {
  nodes: Node[];
  links: Link[];
}
