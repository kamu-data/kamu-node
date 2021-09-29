import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import {SearchComponent} from "./search/search.component";
import {LoginComponent} from "./auth/login/login.component";
import {DatasetComponent} from "./dataset/dataset.component";

const routes: Routes = [
  { path: '', redirectTo: 'search', pathMatch: 'full' },
  { path: 'search', component: SearchComponent },
  { path: 'login', component: LoginComponent },
  { path: 'dataset',
    component: DatasetComponent,
    children: [
      { path: '', redirectTo: 'dataset', pathMatch: 'full' },
      { path: ':id', component: DatasetComponent }
    ]
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
