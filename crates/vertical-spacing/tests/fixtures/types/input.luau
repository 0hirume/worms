type First = string

type Second = number
export type PublicFirst = First
export type PublicSecond = Second
type Record = {
long_value_name: string,
another_value_name: number,
}
type Other = {
long_value_name: number,
another_value_name: string,
}
